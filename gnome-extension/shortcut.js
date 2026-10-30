// SPDX-License-Identifier: GPL-3.0-or-later

import Gio from 'gi://Gio';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const KEY = 'toggle-history';
const FREED_KEY = 'freed-shortcuts';
const SHELL_SCHEMA = 'org.gnome.shell.keybindings';
const MESSAGE_TRAY_KEY = 'toggle-message-tray';

export class Shortcut {
    constructor(settings, onActivate) {
        this._settings = settings;
        this._shellSettings = new Gio.Settings({schema_id: SHELL_SCHEMA});
        this._freeConflicts();
        this._changedId = settings.connect(`changed::${KEY}`, () => {
            this._restoreConflicts();
            this._freeConflicts();
        });
        const action = Main.wm.addKeybinding(KEY, settings, Meta.KeyBindingFlags.NONE,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW, onActivate);
        if (action === Meta.KeyBindingAction.NONE)
            console.warn('WinClip: the history shortcut is already used by something else');
    }

    destroy() {
        this._settings.disconnect(this._changedId);
        Main.wm.removeKeybinding(KEY);
        this._restoreConflicts();
    }

    // FREED_KEY is persisted so the shortcut is given back even if GNOME Shell crashed
    // before disable() could run.
    _freeConflicts() {
        const ours = this._settings.get_strv(KEY);
        const tray = this._shellSettings.get_strv(MESSAGE_TRAY_KEY);
        const taken = tray.filter(accel => ours.includes(accel));
        if (taken.length === 0)
            return;
        this._shellSettings.set_strv(MESSAGE_TRAY_KEY, tray.filter(accel => !taken.includes(accel)));
        const freed = this._settings.get_strv(FREED_KEY);
        this._settings.set_strv(FREED_KEY, [...freed, ...taken.filter(accel => !freed.includes(accel))]);
    }

    _restoreConflicts() {
        const freed = this._settings.get_strv(FREED_KEY);
        if (freed.length === 0)
            return;
        const tray = this._shellSettings.get_strv(MESSAGE_TRAY_KEY);
        this._shellSettings.set_strv(MESSAGE_TRAY_KEY, [...tray, ...freed.filter(accel => !tray.includes(accel))]);
        this._settings.set_strv(FREED_KEY, []);
    }
}
