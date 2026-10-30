// SPDX-License-Identifier: GPL-3.0-or-later

import Adw from 'gi://Adw';
import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

import {DaemonClient} from './daemonClient.js';
import {LimitsGroup} from './prefsLimits.js';
import {createShortcutGroup} from './prefsShortcut.js';

export default class WinClipPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const showError = title => window.add_toast(new Adw.Toast({title, timeout: 0}));
        const limits = new LimitsGroup(new DaemonClient(), showError);

        const page = new Adw.PreferencesPage();
        page.add(createShortcutGroup(this.getSettings()));
        page.add(limits.widget);
        window.add(page);

        window.connect('close-request', () => {
            limits.flush();
            return false;
        });
        limits.load().catch(e => showError(`Couldn't reach WinClip's background service: ${e.message}`));
    }
}
