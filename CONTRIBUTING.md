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

Requires GNOME Shell 46 and Node.js 18+ (for linting only).

```sh
cd gnome-extension
npm ci
npm run lint
```

To try your changes, run `make install` from the repository root, then log out and
back in (Wayland can't restart the Shell in place). The preferences window is the
exception: it loads fresh each time it opens, so reopening it is enough.

For faster iteration, run a nested Shell instead of logging out:
`dbus-run-session -- gnome-shell --nested --wayland`.

Before a release, run the [manual test checklist](docs/manual-testing.md).

## Releasing

1. Update the version in `daemon/Cargo.toml` (then run `cargo check` to update
   `Cargo.lock`) and `version-name` in `gnome-extension/metadata.json`.
2. Add the changes to `CHANGELOG.md`.
3. Run the [manual test checklist](docs/manual-testing.md).
4. After merging, tag the commit on `main` and push the tag:
   `git tag v0.2.0 && git push origin v0.2.0`. Or publish a release with a new `v0.2.0`
   tag on GitHub's Releases page; CI then attaches the files to it.

CI then checks that the tag matches both version numbers, runs the tests, builds the
archive and publishes the GitHub Release. Run `make dist` to build the same archive locally.

## Pull requests

- Keep PRs focused; one logical change per PR.
- CI must pass (`cargo fmt`, `clippy`, `cargo test`, ESLint).
- Business logic belongs in the daemon, not in a frontend — see ARCHITECTURE.md.
- If you change anything that touches clipboard capture, describe which apps you
  tested with (e.g. Firefox, GNOME Text Editor, LibreOffice, a password manager).

## Reporting bugs

Use the issue templates. Please include your distro, GNOME Shell version, and session
type (`echo $XDG_SESSION_TYPE`).
