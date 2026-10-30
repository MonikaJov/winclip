// SPDX-License-Identifier: GPL-3.0-or-later

import Adw from 'gi://Adw';
import Gdk from 'gi://Gdk';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

const KEY = 'toggle-history';
const ALLOWED_MODIFIERS = Gdk.ModifierType.CONTROL_MASK | Gdk.ModifierType.SHIFT_MASK |
    Gdk.ModifierType.ALT_MASK | Gdk.ModifierType.SUPER_MASK;
const REQUIRED_MODIFIERS = Gdk.ModifierType.CONTROL_MASK | Gdk.ModifierType.ALT_MASK |
    Gdk.ModifierType.SUPER_MASK;
const GNOME_SHORTCUT_SCHEMAS = [
    'org.gnome.desktop.wm.keybindings',
    'org.gnome.shell.keybindings',
    'org.gnome.mutter.keybindings',
    'org.gnome.mutter.wayland.keybindings',
    'org.gnome.settings-daemon.plugins.media-keys',
];
// WinClip's extension borrows this one itself (see shortcut.js), so it's not a conflict.
const BORROWABLE = {schema: 'org.gnome.shell.keybindings', key: 'toggle-message-tray'};
const HINT = 'It must include Ctrl, Alt or Super.\nEsc cancels, Backspace restores Super+V.';

export function createShortcutGroup(settings) {
    const label = new Gtk.ShortcutLabel({
        accelerator: currentShortcut(settings),
        disabled_text: 'None',
        valign: Gtk.Align.CENTER,
    });
    settings.connect(`changed::${KEY}`, () => (label.accelerator = currentShortcut(settings)));

    const row = new Adw.ActionRow({
        title: 'Show clipboard history',
        subtitle: 'Click to change',
        activatable: true,
    });
    row.add_suffix(label);
    row.connect('activated', () => captureShortcut(row, settings));

    const group = new Adw.PreferencesGroup({title: 'Shortcut'});
    group.add(row);
    return group;
}

function currentShortcut(settings) {
    return settings.get_strv(KEY)[0] ?? '';
}

function captureShortcut(parent, settings) {
    const status = new Adw.StatusPage({
        title: 'Press the new shortcut',
        description: HINT,
        icon_name: 'preferences-desktop-keyboard-shortcuts-symbolic',
    });
    const content = new Adw.ToolbarView({content: status});
    content.add_top_bar(new Adw.HeaderBar());
    const window = new Adw.Window({
        title: 'Set shortcut',
        modal: true,
        transient_for: parent.get_root(),
        default_width: 420,
        content,
    });

    const keys = new Gtk.EventControllerKey();
    keys.connect('key-pressed', (_keys, keyval, keycode, state) => {
        let mods = state & ALLOWED_MODIFIERS;
        if (mods === 0 && keyval === Gdk.KEY_Escape) {
            window.close();
            return Gdk.EVENT_STOP;
        }
        if (mods === 0 && keyval === Gdk.KEY_BackSpace) {
            settings.reset(KEY);
            window.close();
            return Gdk.EVENT_STOP;
        }

        const lower = Gdk.keyval_to_lower(keyval);
        if (lower !== keyval)
            mods |= Gdk.ModifierType.SHIFT_MASK;
        if ((mods & REQUIRED_MODIFIERS) === 0 || !Gtk.accelerator_valid(lower, mods))
            return Gdk.EVENT_STOP;

        const owner = findGnomeOwner(lower, mods);
        if (owner) {
            status.description = `${Gtk.accelerator_get_label(lower, mods)} is already used by ` +
                `GNOME (${owner}). Choose another.\n\n${HINT}`;
            return Gdk.EVENT_STOP;
        }
        settings.set_strv(KEY, [Gtk.accelerator_name_with_keycode(null, lower, keycode, mods)]);
        window.close();
        return Gdk.EVENT_STOP;
    });
    window.add_controller(keys);
    window.present();
}

function findGnomeOwner(keyval, mods) {
    const source = Gio.SettingsSchemaSource.get_default();
    for (const id of GNOME_SHORTCUT_SCHEMAS) {
        const schema = source.lookup(id, true);
        if (!schema)
            continue;
        const settings = new Gio.Settings({settings_schema: schema});
        for (const key of schema.list_keys()) {
            if (id === BORROWABLE.schema && key === BORROWABLE.key)
                continue;
            const value = settings.get_value(key);
            const accels = value.get_type_string() === 'as' ? value.deepUnpack()
                : value.get_type_string() === 's' ? [value.deepUnpack()] : [];
            if (accels.some(accel => sameShortcut(accel, keyval, mods)))
                return schema.get_key(key).get_summary() ?? key;
        }
    }
    return null;
}

function sameShortcut(accel, keyval, mods) {
    const [ok, otherKeyval, otherMods] = Gtk.accelerator_parse(accel);
    return ok && Gdk.keyval_to_lower(otherKeyval) === keyval &&
        (otherMods & ALLOWED_MODIFIERS) === mods;
}
