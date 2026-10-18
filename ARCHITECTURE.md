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
- Exposes the session D-Bus service `org.winclip.Daemon`. This is the seam every
  frontend talks to:
  - Phase 1: `AddItem`, `GetHistory`, `DeleteItem`, `ClearHistory`, `Search(query)`.
  - Phase 1.5: `PinItem`, `UnpinItem`, `PasteItem(id)`.
  - Signal: `HistoryChanged`.
- Applies all business rules in one place so every frontend behaves the same: dedup,
  history size/age cap, and (from Phase 1.5) pinned-item protection.
- Started on demand by a systemd `--user` unit.

### `gnome-extension` (GJS, GNOME 45+, ESM)

- Polls `St.Clipboard` every ~500 ms–1 s (there is no clipboard-changed signal).
- Reads `text/plain` and `text/html` (Phase 1), forwards new items to the daemon.
- Drops anything carrying a password-manager hint (e.g.
  `x-kde-passwordManagerHint`) **before** it reaches the daemon. This is best-effort:
  there is no universal standard for the hint.
- Registers the hotkey (default `Super+V`) and renders the overlay as Shell UI.
- Phase 1 selection only puts the item back on the clipboard; the user presses Ctrl+V.

## Rules of thumb for contributors

- **Business logic goes in the daemon**, not in a frontend. If two frontends would need
  the same rule, it belongs behind D-Bus.
- **Sensitive content is filtered at capture time** in the frontend, so it never
  crosses the D-Bus boundary or touches disk.
- **Target GNOME 45+ (ESM extensions)** as the floor; GNOME Shell APIs change between
  releases, so keep `metadata.json`'s `shell-version` list to versions actually tested.
