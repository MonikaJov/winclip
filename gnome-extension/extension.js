// SPDX-License-Identifier: GPL-3.0-or-later

import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

import {ClipboardWatcher} from './clipboardWatcher.js';
import {DaemonClient, isCancelled} from './daemonClient.js';
import {HistoryPopup} from './historyPopup.js';
import {Shortcut} from './shortcut.js';

export default class WinClipExtension extends Extension {
    enable() {
        this._daemon = new DaemonClient();
        this._watcher = new ClipboardWatcher(({text, html}) => this._daemon.addItem(text, html));
        this._shortcut = new Shortcut(this.getSettings(), () => this._togglePopup());
        this._popup = null;
        this._loading = false;
    }

    disable() {
        this._shortcut.destroy();
        this._shortcut = null;
        this._popup?.close();
        this._popup = null;
        this._watcher.destroy();
        this._watcher = null;
        this._daemon.destroy();
        this._daemon = null;
    }

    async _togglePopup() {
        if (this._popup) {
            this._popup.close();
            return;
        }
        if (this._loading)
            return;

        this._loading = true;
        const daemon = this._daemon;
        let items;
        try {
            items = await daemon.getHistory();
        } catch (e) {
            if (!isCancelled(e))
                Main.notifyError('WinClip', `Couldn't load the clipboard history: ${e.message}`);
            return;
        } finally {
            this._loading = false;
        }
        if (daemon !== this._daemon)
            return;

        const popup = new HistoryPopup(items, {
            onPick: id => this._pick(id),
            onDelete: id => this._delete(id),
        });
        popup.connect('closed', () => {
            if (this._popup === popup)
                this._popup = null;
        });
        if (popup.open()) {
            this._popup = popup;
        } else {
            console.warn('WinClip: could not open the history popup, another dialog has the keyboard');
            popup.destroy();
        }
    }

    async _pick(id) {
        this._popup?.close();
        try {
            // Plain text only: GNOME 46 can't offer several formats from an extension.
            const {text} = await this._daemon.getItem(id);
            St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, text);
        } catch (e) {
            if (!isCancelled(e))
                Main.notifyError('WinClip', `Couldn't copy that item: ${e.message}`);
        }
    }

    async _delete(id) {
        try {
            await this._daemon.deleteItem(id);
        } catch (e) {
            if (!isCancelled(e))
                console.warn(`WinClip: deleting an item failed: ${e.message}`);
        }
    }
}
