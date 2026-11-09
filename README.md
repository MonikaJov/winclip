# WinClip

Clipboard history for Linux, like Win+V on Windows. Press **Super+V**, pick something
you copied earlier and paste it again.

**Status:** early development. Text clipboard history works on GNOME 46 (Wayland).
Images, pinning and automatic pasting are planned.

How it works: [ARCHITECTURE.md](ARCHITECTURE.md). Want to help? See
[CONTRIBUTING.md](CONTRIBUTING.md).

## Requirements

GNOME Shell 46 on a 64-bit PC (tested on Ubuntu 24.04, Wayland session).

## Install

WinClip installs into your home folder, so you don't need `sudo`.

```sh
curl -fsSL https://github.com/MonikaJov/winclip/releases/latest/download/get.sh | sh
```

Log out and back in (GNOME only picks up new extensions at login), then press Super+V.

To update, run the same command again and log out and back in. Your history and
settings stay.

### Install manually

If you'd rather not pipe a script into `sh`, download `winclip-x86_64.tar.gz` from the
[Releases page](https://github.com/MonikaJov/winclip/releases), then:

```sh
tar xzf winclip-x86_64.tar.gz
cd winclip-*/
./install.sh
```

Then log out and back in.

### Build from source

You'll need [Rust](https://rustup.rs) (stable), a C compiler
(`sudo apt install build-essential`), `make` and `glib-compile-schemas` (package
`libglib2.0-bin`, included with GNOME).

```sh
git clone https://github.com/MonikaJov/winclip.git
cd winclip
make install
```

Then log out and back in.

### What gets installed

| What | Where |
|---|---|
| Background service (daemon) | `~/.local/bin/winclip-daemon` |
| Uninstaller | `~/.local/bin/winclip-uninstall` |
| Start-on-demand files | `~/.local/share/dbus-1/services/`, `~/.config/systemd/user/` |
| GNOME extension | `~/.local/share/gnome-shell/extensions/winclip@monikajov.github.io/` |

The daemon doesn't autostart. It starts the first time the extension needs it.

## Usage

Copy text as usual. To reuse something:

| Key | Action |
|---|---|
| **Super+V** | Open the history (newest first) |
| **↑ / ↓** | Move through items |
| **Enter** or click | Copy the item and close; then paste with **Ctrl+V** |
| **Delete** | Remove the selected item |
| **Esc** or **Super+V** | Close |

Pasting from the history gives **plain text**: GNOME 46 doesn't let extensions offer
formatted (HTML) and plain text at the same time.

**Super+V** is normally GNOME's shortcut for the notification list. While WinClip is
enabled it takes over Super+V (notifications stay on **Super+M**) and gives it back
when disabled.

## Settings

Open **Extensions** → WinClip's gear icon, or run
`gnome-extensions prefs winclip@monikajov.github.io`.

- **Shortcut**: click the row and press a new combination (must include Ctrl, Alt or
  Super). Backspace restores Super+V.
- **Items to keep** (default 25), **Keep items for (days)** (default 0 = no age limit),
  **Largest item (KB)** (default 1024).

Lowering a limit **permanently deletes** the items that no longer fit.

## Uninstall

```sh
~/.local/bin/winclip-uninstall
```

This disables the extension, gives Super+V back to GNOME and removes the installed
files. Your history stays; delete it with `rm -rf ~/.local/share/winclip`.

## A note on sensitive content

WinClip skips clipboard items that carry a password-manager "don't record this" hint.
There is no universal standard for that hint, so this is **best-effort, not a
guarantee**. Don't rely on it for high-stakes secrets until stronger protections
(encryption at rest, auto-expiry, per-app exclusions) land.

Your history is private from other user accounts and from most sandboxed (Flatpak/Snap)
apps, but **any regular program running as you can read it**, as can sandboxed apps that
were given access to your home folder. See the security model in
[ARCHITECTURE.md](ARCHITECTURE.md#security-model) for details.

## Troubleshooting

**Super+V opens notifications instead of WinClip.** Check that the extension is running:
`gnome-extensions info winclip@monikajov.github.io` should say `State: ACTIVE`. If WinClip
was removed without being disabled first, give Super+V back to GNOME with:

```sh
gsettings reset org.gnome.shell.keybindings toggle-message-tray
```

**Something doesn't work.** Logs never contain clipboard contents, so they're safe to share:

```sh
journalctl --user -b -o cat /usr/bin/gnome-shell | grep -i winclip   # extension
journalctl --user -b -u winclip-daemon                                 # daemon
```

## License

[GPL-3.0-or-later](LICENSE).
