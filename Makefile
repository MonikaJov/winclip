# SPDX-License-Identifier: GPL-3.0-or-later

VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' daemon/Cargo.toml)
RELEASE := winclip-$(VERSION)
STAGE := dist/$(RELEASE)
ARCHIVE := winclip-x86_64.tar.gz
DAEMON_BIN := daemon/target/release/winclip-daemon
EXTENSION_FILES := metadata.json stylesheet.css extension.js clipboardWatcher.js \
	daemonClient.js historyPopup.js shortcut.js prefs.js prefsLimits.js prefsShortcut.js
EXTENSION_SCHEMA := schemas/org.gnome.shell.extensions.winclip.gschema.xml

.PHONY: all build stage dist install uninstall

all: build

build:
	cargo build --release --locked --manifest-path daemon/Cargo.toml

stage: build
	rm -rf $(STAGE)
	install -Dm755 $(DAEMON_BIN) $(STAGE)/bin/winclip-daemon
	install -Dm644 -t $(STAGE)/data daemon/data/*.in
	install -Dm644 -t $(STAGE)/extension $(addprefix gnome-extension/,$(EXTENSION_FILES))
	install -Dm644 -t $(STAGE)/extension/schemas gnome-extension/$(EXTENSION_SCHEMA)
	glib-compile-schemas $(STAGE)/extension/schemas
	install -Dm755 -t $(STAGE) packaging/install.sh packaging/uninstall.sh
	install -Dm644 -t $(STAGE) README.md LICENSE

dist: stage
	tar -C dist -czf dist/$(ARCHIVE) $(RELEASE)
	cd dist && sha256sum $(ARCHIVE) > SHA256SUMS

install: stage
	$(STAGE)/install.sh

uninstall:
	packaging/uninstall.sh
