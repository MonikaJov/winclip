# Contributing to WinClip

Thanks for your interest! WinClip is early-stage; see [ARCHITECTURE.md](ARCHITECTURE.md)
for why the code is split the way it is.

## License

WinClip is licensed under the GNU General Public License v3.0 or later
(`GPL-3.0-or-later`). By submitting a contribution you agree that it is licensed under
the same terms. New source files should start with an SPDX header:

```
// SPDX-License-Identifier: GPL-3.0-or-later
```

## Development setup

### Daemon (Rust)

Requires a stable Rust toolchain (`rustup`).

```sh
cd daemon
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Code style is enforced by the tools above: the 2024 edition, `cargo fmt`, clippy's
`pedantic` lints, and no `unsafe` code (see the `[lints]` section in
`daemon/Cargo.toml`). CI fails on any warning. Beyond that: avoid `as` for number
conversions (use `From`/`TryFrom`), don't `unwrap()` outside tests, and comment only
to explain *why*, not *what* the code does.

### GNOME extension (GJS)

Requires GNOME Shell 45+ and Node.js 18+ (for linting only).

```sh
cd gnome-extension
npm ci
npm run lint
```

To try the extension locally, symlink it into your extensions directory and log out
and back in (Wayland can't restart the Shell in place):

```sh
ln -s "$PWD" ~/.local/share/gnome-shell/extensions/winclip@monikajov.github.io
gnome-extensions enable winclip@monikajov.github.io
```

For faster iteration, run a nested Shell instead of logging out:
`dbus-run-session -- gnome-shell --nested --wayland` (GNOME 45–48) or
`dbus-run-session -- gnome-shell --devkit` (GNOME 49+).

## Pull requests

- Keep PRs focused; one logical change per PR.
- CI must pass (`cargo fmt`, `clippy`, `cargo test`, ESLint).
- Business logic belongs in the daemon, not in a frontend — see ARCHITECTURE.md.
- If you change anything that touches clipboard capture, describe which apps you
  tested with (e.g. Firefox, GNOME Text Editor, LibreOffice, a password manager).

## Reporting bugs

Use the issue templates. Please include your distro, GNOME Shell version, and session
type (`echo $XDG_SESSION_TYPE`).
