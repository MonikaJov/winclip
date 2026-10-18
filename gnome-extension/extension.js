// SPDX-License-Identifier: GPL-3.0-or-later

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

import {ClipboardWatcher} from './clipboardWatcher.js';
import {DaemonClient} from './daemonClient.js';

export default class WinClipExtension extends Extension {
    enable() {
        this._daemon = new DaemonClient();
        this._watcher = new ClipboardWatcher(({text, html}) => this._daemon.addItem(text, html));
    }

    disable() {
        this._watcher.destroy();
        this._watcher = null;
        this._daemon.destroy();
        this._daemon = null;
    }
}
