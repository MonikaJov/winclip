// SPDX-License-Identifier: GPL-3.0-or-later

import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';
import Pango from 'gi://Pango';
import St from 'gi://St';
import {ensureActorVisibleInScrollView} from 'resource:///org/gnome/shell/misc/animationUtils.js';
import * as ModalDialog from 'resource:///org/gnome/shell/ui/modalDialog.js';

export const HistoryPopup = GObject.registerClass(
class HistoryPopup extends ModalDialog.ModalDialog {
    _init(items, {onPick, onDelete}) {
        super._init({styleClass: 'winclip-popup'});
        this._onPick = onPick;
        this._onDelete = onDelete;

        this._list = new St.BoxLayout({vertical: true, style_class: 'winclip-list'});
        this._scrollView = new St.ScrollView({
            style_class: 'winclip-scroll',
            hscrollbar_policy: St.PolicyType.NEVER,
            child: this._list,
        });
        this._emptyLabel = new St.Label({
            text: 'Clipboard history is empty',
            style_class: 'winclip-empty',
        });
        this.contentLayout.add_child(this._scrollView);
        this.contentLayout.add_child(this._emptyLabel);

        for (const item of items)
            this._list.add_child(this._createRow(item));
        this._updateEmptyState();
        const first = this._list.get_first_child();
        if (first)
            this.setInitialKeyFocus(first);
    }

    vfunc_key_press_event(event) {
        if (event.get_key_symbol() === Clutter.KEY_Escape) {
            this.close();
            return Clutter.EVENT_STOP;
        }
        return super.vfunc_key_press_event(event);
    }

    _createRow(item) {
        const label = new St.Label({
            text: item.preview.replace(/\s+/g, ' ').trim() || '(whitespace)',
            x_align: Clutter.ActorAlign.START,
            y_align: Clutter.ActorAlign.CENTER,
        });
        label.clutter_text.ellipsize = Pango.EllipsizeMode.END;

        const row = new St.Button({
            child: label,
            style_class: 'winclip-item',
            can_focus: true,
            x_expand: true,
        });
        row.connect('clicked', () => this._onPick(item.id));
        row.connect('key-focus-in', () => ensureActorVisibleInScrollView(this._scrollView, row));
        row.connect('key-press-event', (_row, event) => {
            if (event.get_key_symbol() !== Clutter.KEY_Delete)
                return Clutter.EVENT_PROPAGATE;
            this._deleteRow(row, item.id);
            return Clutter.EVENT_STOP;
        });
        return row;
    }

    _deleteRow(row, id) {
        const neighbour = row.get_next_sibling() ?? row.get_previous_sibling();
        this._onDelete(id);
        row.destroy();
        this._updateEmptyState();
        neighbour?.grab_key_focus();
    }

    _updateEmptyState() {
        const empty = this._list.get_n_children() === 0;
        this._scrollView.visible = !empty;
        this._emptyLabel.visible = empty;
    }
});
