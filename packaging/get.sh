#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu

# Wrapped in a function so a download cut off halfway through `curl | sh` runs nothing.
main() {
    machine=$(uname -m)
    if [ "$machine" != x86_64 ]; then
        echo "WinClip only has builds for x86_64 PCs, and this one is $machine." >&2
        exit 1
    fi

    base=https://github.com/MonikaJov/winclip/releases/latest/download
    archive=winclip-x86_64.tar.gz
    work=$(mktemp -d)
    trap 'rm -rf "$work"' EXIT

    curl -fsSL -o "$work/$archive" "$base/$archive"
    curl -fsSL -o "$work/SHA256SUMS" "$base/SHA256SUMS"
    (cd "$work" && sha256sum --check --quiet SHA256SUMS)

    tar -xzf "$work/$archive" -C "$work"
    sh "$work"/winclip-*/install.sh
}

main
