# SPDX-License-Identifier: GPL-3.0-or-later
#
# Installs into the current user's home only; never needs root.

PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
XDG_DATA_HOME ?= $(HOME)/.local/share
XDG_CONFIG_HOME ?= $(HOME)/.config
DBUS_SERVICE_DIR ?= $(XDG_DATA_HOME)/dbus-1/services
SYSTEMD_USER_DIR ?= $(XDG_CONFIG_HOME)/systemd/user

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
ifeq ($(DESTDIR),)
	systemctl --user daemon-reload
	busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
	-systemctl --user try-restart $(SYSTEMD_UNIT)
endif

uninstall:
ifeq ($(DESTDIR),)
	-systemctl --user stop $(SYSTEMD_UNIT)
endif
	rm -f $(DESTDIR)$(BINDIR)/winclip-daemon
	rm -f $(DESTDIR)$(DBUS_SERVICE_DIR)/$(DBUS_SERVICE)
	rm -f $(DESTDIR)$(SYSTEMD_USER_DIR)/$(SYSTEMD_UNIT)
ifeq ($(DESTDIR),)
	systemctl --user daemon-reload
	busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
endif
	@echo "Clipboard history kept in $(XDG_DATA_HOME)/winclip; delete it to remove your history."
