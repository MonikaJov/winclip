# WinClip

*Working title.* A lightweight Linux clipboard history manager aiming for parity with
Windows' Win+V: a global hotkey, a fast overlay of recent clipboard items, and
click/keyboard to re-paste.

**Status:** early development (Phase 0 — project setup). Nothing is usable yet.

- [ARCHITECTURE.md](ARCHITECTURE.md) — why it's a GNOME Shell extension plus a daemon
- [CONTRIBUTING.md](CONTRIBUTING.md) — dev setup and PR guidelines

## A note on sensitive content

WinClip skips clipboard items that carry a password-manager "don't record this" hint.
There is no universal standard for that hint, so this is **best-effort, not a
guarantee**. Don't rely on it for high-stakes secrets until stronger protections
(encryption at rest, auto-expiry, per-app exclusions) land.

## License

[GPL-3.0-or-later](LICENSE).
