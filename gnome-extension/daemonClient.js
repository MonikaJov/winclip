// SPDX-License-Identifier: GPL-3.0-or-later

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

const BUS_NAME = 'org.winclip.Daemon';
const OBJECT_PATH = '/org/winclip/Daemon1';
const INTERFACE = 'org.winclip.Daemon1';
const TOO_LARGE = 'org.winclip.Daemon1.Error.TooLarge';

export class DaemonClient {
    constructor() {
        this._cancellable = new Gio.Cancellable();
        this._failing = false;
    }

    destroy() {
        this._cancellable.cancel();
    }

    addItem(text, html) {
        if (text.includes('\0') || html.includes('\0'))
            return;
        Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, INTERFACE, 'AddItem',
            new GLib.Variant('(ss)', [text, html]),
            new GLib.VariantType('(t)'), Gio.DBusCallFlags.NONE, -1, this._cancellable,
            (connection, result) => {
                try {
                    connection.call_finish(result);
                    this._failing = false;
                } catch (e) {
                    if (e.matches(Gio.IOErrorEnum, Gio.IOErrorEnum.CANCELLED) ||
                        Gio.DBusError.get_remote_error(e) === TOO_LARGE)
                        return;
                    if (!this._failing)
                        console.warn(`WinClip: saving clipboard items failed: ${e.message}`);
                    this._failing = true;
                }
            });
    }
}
