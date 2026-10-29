# SPDX-License-Identifier: GPL-3.0-or-later
#
# Installs into the current user's home only; never needs root.

PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
XDG_DATA_HOME ?= $(HOME)/.local/share
XDG_CONFIG_HOME ?= $(HOME)/.config
DBUS_SERVICE_DIR ?= $(XDG_DATA_HOME)/dbus-1/services
SYSTEMD_USER_DIR ?= $(XDG_CONFIG_HOME)/systemd/user
EXTENSION_UUID := winclip@monikajov.github.io
EXTENSION_DIR ?= $(XDG_DATA_HOME)/gnome-shell/extensions/$(EXTENSION_UUID)
EXTENSION_FILES := metadata.json stylesheet.css extension.js clipboardWatcher.js \
	daemonClient.js historyPopup.js shortcut.js
EXTENSION_SCHEMA := schemas/org.gnome.shell.extensions.winclip.gschema.xml

DAEMON_BIN := daemon/target/release/winclip-daemon
DBUS_SERVICE := org.winclip.Daemon.service
SYSTEMD_UNIT := winclip-daemon.service

.PHONY: all build install uninstall

all: build

build:
	cargo build --release --manifest-path daemon/Cargo.toml

install: build
	install -Dm755 $(DAEMON_BIN) $(DESTDIR)$(BINDIR)/winclip-daemon
	install -d $(DESTDIR)$(DBUS_SERVICE_DIR) $(DESTDIR)$(SYSTEMD_USER_DIR)
	sed 's|@BINDIR@|$(BINDIR)|g' daemon/data/$(DBUS_SERVICE).in > $(DESTDIR)$(DBUS_SERVICE_DIR)/$(DBUS_SERVICE)
	sed 's|@BINDIR@|$(BINDIR)|g' daemon/data/$(SYSTEMD_UNIT).in > $(DESTDIR)$(SYSTEMD_USER_DIR)/$(SYSTEMD_UNIT)
	install -Dm644 -t $(DESTDIR)$(EXTENSION_DIR) $(addprefix gnome-extension/,$(EXTENSION_FILES))
	install -Dm644 -t $(DESTDIR)$(EXTENSION_DIR)/schemas gnome-extension/$(EXTENSION_SCHEMA)
	glib-compile-schemas $(DESTDIR)$(EXTENSION_DIR)/schemas
ifeq ($(DESTDIR),)
	systemctl --user daemon-reload
	busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
	-systemctl --user try-restart $(SYSTEMD_UNIT)
endif

uninstall:
ifeq ($(DESTDIR),)
	-gnome-extensions disable $(EXTENSION_UUID)
	-systemctl --user stop $(SYSTEMD_UNIT)
endif
	rm -f $(addprefix $(DESTDIR)$(EXTENSION_DIR)/,$(EXTENSION_FILES) $(EXTENSION_SCHEMA) schemas/gschemas.compiled)
	-rmdir $(DESTDIR)$(EXTENSION_DIR)/schemas $(DESTDIR)$(EXTENSION_DIR)
	rm -f $(DESTDIR)$(BINDIR)/winclip-daemon
	rm -f $(DESTDIR)$(DBUS_SERVICE_DIR)/$(DBUS_SERVICE)
	rm -f $(DESTDIR)$(SYSTEMD_USER_DIR)/$(SYSTEMD_UNIT)
ifeq ($(DESTDIR),)
	systemctl --user daemon-reload
	busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
endif
	@echo "Clipboard history kept in $(XDG_DATA_HOME)/winclip; delete it to remove your history."
