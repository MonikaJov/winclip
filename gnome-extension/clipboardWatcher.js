// SPDX-License-Identifier: GPL-3.0-or-later

import Gio from 'gi://Gio';
import Meta from 'gi://Meta';

const CLIPBOARD = Meta.SelectionType.SELECTION_CLIPBOARD;
const SENSITIVE_MIMETYPES = ['x-kde-passwordManagerHint'];
const TEXT_MIMETYPES = ['text/plain;charset=utf-8', 'UTF8_STRING', 'text/plain'];
const HTML_MIMETYPE = 'text/html';

const MAX_READ_BYTES = 10 * 1024 * 1024;

export class ClipboardWatcher {
    constructor(onCopy) {
        this._onCopy = onCopy;
        this._selection = global.display.get_selection();
        this._cancellable = new Gio.Cancellable();
        this._latestRead = 0;
        this._ownerChangedId = this._selection.connect('owner-changed', (_selection, type) => {
            if (type === CLIPBOARD)
                this._read().catch(e => console.error(`WinClip: reading the clipboard failed: ${e.message}`));
        });
    }

    destroy() {
        this._selection.disconnect(this._ownerChangedId);
        this._cancellable.cancel();
        this._latestRead++;
        this._onCopy = null;
    }

    async _read() {
        const readId = ++this._latestRead;
        const mimetypes = this._selection.get_mimetypes(CLIPBOARD);
        if (mimetypes.some(m => SENSITIVE_MIMETYPES.includes(m)))
            return;
        const textMimetype = TEXT_MIMETYPES.find(m => mimetypes.includes(m));
        if (!textMimetype)
            return;

        const textBytes = await this._transfer(textMimetype);
        if (!textBytes || readId !== this._latestRead)
            return;
        
        const htmlBytes = mimetypes.includes(HTML_MIMETYPE) ? await this._transfer(HTML_MIMETYPE) : null;
        if (readId !== this._latestRead)
            return;

        const text = new TextDecoder('utf-8').decode(textBytes);
        if (text)
            this._onCopy({text, html: decodeHtml(htmlBytes)});
    }

    _transfer(mimetype) {
        const stream = Gio.MemoryOutputStream.new_resizable();
        return new Promise((resolve, reject) => {
            this._selection.transfer_async(CLIPBOARD, mimetype, MAX_READ_BYTES + 1, stream,
                this._cancellable, (selection, result) => {
                    try {
                        selection.transfer_finish(result);
                        stream.close(null);
                        const bytes = stream.steal_as_bytes().get_data() ?? new Uint8Array();
                        resolve(bytes.length > MAX_READ_BYTES ? null : bytes);
                    } catch (e) {
                        if (e.matches(Gio.IOErrorEnum, Gio.IOErrorEnum.CANCELLED))
                            resolve(null);
                        else
                            reject(e);
                    }
                });
        });
    }
}

function decodeHtml(data) {
    if (!data || data.length === 0)
        return '';
    if (data.length >= 2 && data[0] === 0xff && data[1] === 0xfe)
        return new TextDecoder('utf-16le').decode(data.subarray(2));
    if (data.length >= 2 && data[0] === 0xfe && data[1] === 0xff)
        return new TextDecoder('utf-16be').decode(data.subarray(2));
    return new TextDecoder('utf-8').decode(data);
}
