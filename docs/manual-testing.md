# Manual testing

The daemon has automated tests (`cargo test`); the GNOME extension can only be tested in a
real session. Run this checklist before a release, on a fresh `make install` followed by
logging out and back in.

## Capture

Copy from each app, then open the history with **Super+V**:

| Source | What to copy | Expected |
|---|---|---|
| Terminal | A command | Appears at the top |
| Firefox / Chrome | Bold text from a web page | Appears; pastes as plain text |
| GNOME Text Editor | Several lines | Shown as one shortened line; pastes in full |
| LibreOffice Writer | A paragraph | Appears |
| Any | Text with emoji and accents (😀, é, ü) | Shown and pasted correctly |
| Any | The same text twice | Appears once, moved to the top |
| Terminal | `seq 1 3000000 \| xclip -selection clipboard` (~20 MB) | Not saved; desktop doesn't freeze |
| Password manager (e.g. KeePassXC) | A password | **Not** saved |
| Browser's saved passwords | A password | Note whether it's saved (browsers may not mark passwords) |

Selecting text without copying (middle-click paste) must **not** add anything.

## Popup

- [ ] **↑ / ↓** move the highlight; the list scrolls with it
- [ ] **Enter** and clicking copy the item and close; **Ctrl+V** pastes it
- [ ] The picked item moves to the top of the history
- [ ] **Delete** removes the item and highlights the next one
- [ ] With an empty history the popup says "Clipboard history is empty"
- [ ] **Esc** and **Super+V** close the popup

## Shortcut

- [ ] While enabled, `gsettings get org.gnome.shell.keybindings toggle-message-tray` shows
      no `<Super>v`; **Super+M** opens notifications
- [ ] `gnome-extensions disable winclip@monikajov.github.io` gives Super+V back to
      notifications; enabling takes it again
- [ ] After locking and unlocking the screen, Super+V still opens WinClip

## Settings

- [ ] The window shows the current shortcut and the limits (25, 0, 1024 by default)
- [ ] Lowering **Items to keep** removes the oldest items about a second later
- [ ] A change made just before closing the window is still saved
- [ ] Changing the shortcut to e.g. **Super+Shift+V** works, and Super+V goes back to
      notifications; **Backspace** in the capture window restores Super+V
- [ ] A shortcut without Ctrl, Alt or Super is ignored; **Esc** cancels

## Daemon

- [ ] After logging out and back in, `journalctl --user -b -u winclip-daemon` shows the
      daemon restarting cleanly (a "lost connection to the session bus" exit at logout
      is expected)
- [ ] `ls -l ~/.local/share/winclip` shows `drwx------` and `history.db` as `-rw-------`

## Afterwards

- [ ] `journalctl --user -b -o cat /usr/bin/gnome-shell | grep -i winclip` shows no errors
- [ ] `make uninstall` removes the installed files and restores Super+V
