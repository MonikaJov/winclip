// SPDX-License-Identifier: GPL-3.0-or-later

import Adw from 'gi://Adw';
import GLib from 'gi://GLib';

// Lowering a limit deletes items immediately, so a value is only sent once the user
// stops changing it; clicking "−" from 25 to 20 must not apply 24, 23, … on the way.
const APPLY_DELAY_MS = 800;

const LIMITS = [
    {property: 'MaxItems', title: 'Items to keep', min: 1, max: 1000, scale: 1},
    {
        property: 'MaxAgeDays',
        title: 'Keep items for (days)',
        subtitle: '0 keeps them until newer items push them out',
        min: 0,
        max: 3650,
        scale: 1,
    },
    {
        property: 'MaxItemBytes',
        title: 'Largest item (KB)',
        subtitle: 'Bigger copies are not saved',
        min: 1,
        max: 10 * 1024,
        scale: 1024,
    },
];

export class LimitsGroup {
    constructor(daemon, onError) {
        this._daemon = daemon;
        this._onError = onError;
        this._pending = new Map();
        this.widget = new Adw.PreferencesGroup({
            title: 'History',
            description: 'Lowering a limit removes the items that no longer fit right away.',
        });
        this._rows = LIMITS.map(limit => {
            const row = Adw.SpinRow.new_with_range(limit.min, limit.max, 1);
            row.title = limit.title;
            row.subtitle = limit.subtitle ?? '';
            row.sensitive = false;
            this.widget.add(row);
            return row;
        });
    }

    async load() {
        for (const [i, limit] of LIMITS.entries()) {
            const row = this._rows[i];
            row.value = await this._daemon.getProperty(limit.property) / limit.scale;
            row.connect('notify::value', () => this._schedule(limit, row.value));
            row.sensitive = true;
        }
    }

    flush() {
        for (const [limit, {sourceId, value}] of this._pending) {
            GLib.source_remove(sourceId);
            this._apply(limit, value);
        }
        this._pending.clear();
    }

    _schedule(limit, value) {
        const pending = this._pending.get(limit);
        if (pending)
            GLib.source_remove(pending.sourceId);
        const sourceId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, APPLY_DELAY_MS, () => {
            this._pending.delete(limit);
            this._apply(limit, value);
            return GLib.SOURCE_REMOVE;
        });
        this._pending.set(limit, {sourceId, value});
    }

    _apply(limit, value) {
        this._daemon.setProperty(limit.property, new GLib.Variant('u', value * limit.scale))
            .catch(e => this._onError(`Couldn't change "${limit.title}": ${e.message}`));
    }
}
