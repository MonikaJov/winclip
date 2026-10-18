# Architecture

This document explains *why* WinClip is split the way it is. Read it before touching
the boundary between the daemon and any frontend.

## The key constraint

**On GNOME + Wayland, only code running inside the GNOME Shell process can reliably
read the clipboard and own a global hotkey.**

- Mutter (GNOME's compositor) does **not** implement the `wlr-data-control` protocol
  that tools like `wl-clipboard` rely on. That protocol is wlroots-specific (Sway,
  Hyprland, etc.). A plain background daemon therefore cannot passively watch the
  clipboard on GNOME Wayland the way it could on X11 or a wlroots compositor.
- A **GNOME Shell extension** (GJS, loaded in-process by `gnome-shell`) has access to
  `St.Clipboard` and can poll it directly. This is how existing tools such as
  Clipboard Indicator work.
- The same privileged position lets the extension register a real global keybinding
  (`Main.wm.addKeybinding`, user-remappable in GNOME Settings) and inject synthetic
  input (`Clutter.VirtualInputDevice`) for auto-paste.
- GNOME does not support `wlr-layer-shell`, so a plain GTK window cannot get a reliable,
  borderless, always-on-top overlay. Rendering the overlay as native Shell UI
  (`St`/`Clutter` actors in the Shell's UI group, the same layer used by the Overview
  and the screenshot UI) is the only approach that behaves like a system-level modal.

**Result:** on GNOME, the Shell extension owns clipboard capture, the hotkey, and the
overlay UI. A separate daemon owns storage and business logic, so future X11 and KDE
frontends reuse it instead of re-implementing history management.

## Components

```
daemon/            Rust background service — the only stateful component
gnome-extension/   GJS extension — GNOME Wayland frontend (Phase 1)
x11-client/        GTK4 app — X11 / other-desktop fallback (Phase 2, not started)
kde-plasmoid/      KDE Plasma integration (Phase 3, not started)
common/            D-Bus interface spec, shared docs (not started)
packaging/         Flatpak, AUR, .deb (not started)
```

### `winclip-daemon` (Rust)

- Owns the SQLite database (`~/.local/share/winclip/history.db`) and, from Phase 1.5,
  cached image files (`~/.local/share/winclip/images/`, content-hash filenames).
- Exposes the session D-Bus service `org.winclip.Daemon` (interface
  `org.winclip.Daemon1`). This is the seam every
  frontend talks to:
  - Phase 1: `AddItem`, `GetHistory`, `GetItem`, `DeleteItem`, `ClearHistory`,
    `Search(query)`.
  - Phase 1.5: `PinItem`, `UnpinItem`, `PasteItem(id)`.
  - Signal: `HistoryChanged`.
  - The full spec is [`common/org.winclip.Daemon1.xml`](common/org.winclip.Daemon1.xml).
- Applies all business rules in one place so every frontend behaves the same: dedup,
  history size/age cap, and (from Phase 1.5) pinned-item protection.
- **Owns the history limits** (max items, max age, max item size). They are D-Bus
  properties stored in the daemon's database, so every frontend shares the same
  settings. Frontend-specific preferences (e.g. the GNOME hotkey) stay in the frontend.
- **Started on demand by D-Bus activation**: a D-Bus service file maps the name
  `org.winclip.Daemon` to a systemd `--user` unit, so the first call from any frontend
  starts the daemon. Frontends never start or stop it themselves.

### `gnome-extension` (GJS, GNOME 45+, ESM)

- Polls `St.Clipboard` every ~500 ms–1 s (there is no clipboard-changed signal).
- Reads `text/plain` and `text/html` (Phase 1), forwards new items to the daemon.
- Drops anything carrying a password-manager hint (e.g.
  `x-kde-passwordManagerHint`) **before** it reaches the daemon. This is best-effort:
  there is no universal standard for the hint.
- Registers the hotkey (default `Super+V`) and renders the overlay as Shell UI.
- Phase 1 selection only puts the item back on the clipboard; the user presses Ctrl+V.

## Security model

*This section describes the design. The daemon is being implemented in Phase 1; each
guarantee below is covered by a test once its code lands.*

**Who can read the history?**

- **Other user accounts:** no. The database and its folder are readable only by their
  owner (permissions `0600` / `0700`).
- **Sandboxed apps (Flatpak, Snap):** no, unless the app has access to the session bus
  name `org.winclip.Daemon` or to `~/.local/share/winclip/`. Note that some Flatpaks
  request full home-folder access (`--filesystem=home` or `host`) when installed; those
  can read the database file without the user being asked about it specifically.
- **Unsandboxed programs running as the same user:** yes. They can call the daemon over
  D-Bus or read the database file directly.

The last point is a limit of desktop Linux, not something WinClip can fix: there is no
security boundary between unsandboxed programs running as the same user. They can read
each other's files and settings, so any secret used to authenticate the extension (a
token, a key) could be read the same way. For this reason the daemon does **not**
authenticate its callers in Phase 1. It does validate every input, since any of these
programs can call it.

One consequence worth knowing: unsandboxed programs can already read the *current*
clipboard on GNOME Wayland, even from the background (e.g. through XWayland with
`xclip`, or `wl-paste`, which briefly creates its own surface to read it; both verified
on GNOME 46). What a clipboard history adds is access to *past* clipboard contents,
which would otherwise be gone once replaced.

**What protects sensitive data today:**

- Items marked by password managers as sensitive are dropped by the frontend at capture
  time and never reach the daemon or disk. This is best-effort; see the README.
- History limits (count, age, size) bound how much is kept.

**Planned (Phase 4):** encryption of the database at rest, auto-expiry, a per-app
exclusion list, and an optional caller check (the daemon asks D-Bus for the caller's
process and only accepts known frontends, such as `gnome-shell`). These raise the bar
against casual snooping; they don't stop a determined program running as the same user.

## Rules of thumb for contributors

- **Business logic goes in the daemon**, not in a frontend. If two frontends would need
  the same rule, it belongs behind D-Bus.
- **Sensitive content is filtered at capture time** in the frontend, so it never
  crosses the D-Bus boundary or touches disk.
- **Target GNOME 45+ (ESM extensions)** as the floor; GNOME Shell APIs change between
  releases, so keep `metadata.json`'s `shell-version` list to versions actually tested.
