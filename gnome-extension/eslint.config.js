// SPDX-License-Identifier: GPL-3.0-or-later

import js from '@eslint/js';

export default [
    js.configs.recommended,
    {
        files: ['**/*.js'],
        languageOptions: {
            ecmaVersion: 2022,
            sourceType: 'module',
            globals: {
                // GJS / GNOME Shell globals
                global: 'readonly',
                log: 'readonly',
                logError: 'readonly',
                print: 'readonly',
                printerr: 'readonly',
                console: 'readonly',
                imports: 'readonly',
                TextEncoder: 'readonly',
                TextDecoder: 'readonly',
            },
        },
        rules: {
            'no-unused-vars': ['error', {argsIgnorePattern: '^_'}],
        },
    },
];
