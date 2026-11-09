#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu

if [ "$(id -u)" -eq 0 ]; then
    echo "Don't run this with sudo." >&2
    exit 1
fi

uuid=winclip@monikajov.github.io
destdir=${DESTDIR:-}
bindir=${BINDIR:-${PREFIX:-$HOME/.local}/bin}
data_home=${XDG_DATA_HOME:-$HOME/.local/share}
config_home=${XDG_CONFIG_HOME:-$HOME/.config}

if [ -z "$destdir" ]; then
    gnome-extensions disable "$uuid" 2>/dev/null || true
    systemctl --user stop winclip-daemon.service || true
    # `disable` puts it on this list, which would override the enabled list on a later install.
    if disabled=$(gsettings get org.gnome.shell disabled-extensions); then
        gsettings set org.gnome.shell disabled-extensions \
            "$(printf '%s' "$disabled" | sed "s/'$uuid', //; s/, '$uuid'//; s/'$uuid'//")"
    fi
fi

rm -rf "$destdir$data_home/gnome-shell/extensions/$uuid"
rm -f "$destdir$bindir/winclip-daemon" "$destdir$bindir/winclip-uninstall"
rm -f "$destdir$data_home/dbus-1/services/org.winclip.Daemon.service"
rm -f "$destdir$config_home/systemd/user/winclip-daemon.service"

if [ -z "$destdir" ]; then
    systemctl --user daemon-reload
    busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
fi

echo "WinClip removed. Your clipboard history is still in $data_home/winclip. Delete that folder if you don't want to keep it."
