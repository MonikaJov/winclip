#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu

if [ "$(id -u)" -eq 0 ]; then
    echo "Don't run this with sudo. WinClip installs into your home folder." >&2
    exit 1
fi

here=$(dirname "$(readlink -f "$0")")
uuid=winclip@monikajov.github.io
destdir=${DESTDIR:-}
bindir=${BINDIR:-${PREFIX:-$HOME/.local}/bin}
data_home=${XDG_DATA_HOME:-$HOME/.local/share}
config_home=${XDG_CONFIG_HOME:-$HOME/.config}
dbus_dir=$destdir$data_home/dbus-1/services
systemd_dir=$destdir$config_home/systemd/user
extension_dir=$destdir$data_home/gnome-shell/extensions/$uuid

refuse() {
    echo "$1 Nothing was installed." >&2
    echo "To install anyway, set WINCLIP_FORCE=1." >&2
    exit 1
}

if [ -z "$destdir" ] && [ -z "${WINCLIP_FORCE:-}" ]; then
    shell_version=$(gnome-shell --version 2>/dev/null | sed 's/[^0-9]*\([0-9]*\).*/\1/') || true
    if [ -z "$shell_version" ]; then
        refuse "GNOME Shell wasn't found, and WinClip only works on GNOME."
    elif ! grep -q "\"$shell_version\"" "$here/extension/metadata.json"; then
        refuse "This version of WinClip doesn't work with GNOME Shell $shell_version."
    fi
fi

escaped_bindir=$(printf '%s' "$bindir" | sed 's/[&|\\]/\\&/g')
fill_template() {
    sed "s|@BINDIR@|$escaped_bindir|g" "$here/data/$1.in" > "$2/$1"
}

enable_extension() {
    current=$(gsettings get org.gnome.shell enabled-extensions) || return 0
    case "$current" in
        *"'$uuid'"*) ;;
        *\[\]) gsettings set org.gnome.shell enabled-extensions "['$uuid']" ;;
        *) gsettings set org.gnome.shell enabled-extensions "${current%]}, '$uuid']" ;;
    esac
}

install -Dm755 "$here/bin/winclip-daemon" "$destdir$bindir/winclip-daemon"
install -Dm755 "$here/uninstall.sh" "$destdir$bindir/winclip-uninstall"
mkdir -p "$dbus_dir" "$systemd_dir"
fill_template org.winclip.Daemon.service "$dbus_dir"
fill_template winclip-daemon.service "$systemd_dir"

rm -rf "$extension_dir"
mkdir -p "$extension_dir"
cp -R "$here/extension/." "$extension_dir/"

if [ -z "$destdir" ]; then
    systemctl --user daemon-reload
    busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus ReloadConfig
    systemctl --user try-restart winclip-daemon.service || true
    enable_extension
fi

echo "WinClip installed. Log out and back in, then press Super+V."
case ":$PATH:" in
    *":$bindir:"*) uninstall_command=winclip-uninstall ;;
    *) uninstall_command=$bindir/winclip-uninstall ;;
esac
echo "To uninstall, run $uninstall_command"
