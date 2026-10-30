// SPDX-License-Identifier: GPL-3.0-or-later

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

const BUS_NAME = 'org.winclip.Daemon';
const OBJECT_PATH = '/org/winclip/Daemon1';
const INTERFACE = 'org.winclip.Daemon1';
const PROPERTIES_INTERFACE = 'org.freedesktop.DBus.Properties';
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
        this._call('AddItem', new GLib.Variant('(ss)', [text, html]), '(t)')
            .then(() => (this._failing = false))
            .catch(e => {
                if (isCancelled(e) || Gio.DBusError.get_remote_error(e) === TOO_LARGE)
                    return;
                if (!this._failing)
                    console.warn(`WinClip: saving clipboard items failed: ${e.message}`);
                this._failing = true;
            });
    }

    async getHistory() {
        const [items] = await this._call('GetHistory', new GLib.Variant('(u)', [0]), '(a(tsbx))');
        return items.map(([id, preview, hasHtml]) => ({id, preview, hasHtml}));
    }

    async getItem(id) {
        const [text, html] = await this._call('GetItem', new GLib.Variant('(t)', [id]), '(ss)');
        return {text, html};
    }

    async deleteItem(id) {
        await this._call('DeleteItem', new GLib.Variant('(t)', [id]), '()');
    }

    async getProperty(name) {
        const [value] = await this._call('Get', new GLib.Variant('(ss)', [INTERFACE, name]), '(v)',
            PROPERTIES_INTERFACE);
        return value.unpack();
    }

    async setProperty(name, value) {
        await this._call('Set', new GLib.Variant('(ssv)', [INTERFACE, name, value]), '()',
            PROPERTIES_INTERFACE);
    }

    _call(method, args, replyType, iface = INTERFACE) {
        return new Promise((resolve, reject) => {
            Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, iface, method, args,
                new GLib.VariantType(replyType), Gio.DBusCallFlags.NONE, -1, this._cancellable,
                (connection, result) => {
                    try {
                        resolve(connection.call_finish(result).deepUnpack());
                    } catch (e) {
                        reject(e);
                    }
                });
        });
    }
}

export function isCancelled(e) {
    return e instanceof GLib.Error && e.matches(Gio.IOErrorEnum, Gio.IOErrorEnum.CANCELLED);
}
