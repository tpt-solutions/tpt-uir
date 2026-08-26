# Editor support (TextMate grammar)

`tpt-uir-text.tmLanguage.json` is a TextMate grammar for the textual TPT-UIR
format (`*.uir` / `*.txt` produced by `tpt-uir-cli print` or `convert`). It
highlights block labels (`^bb0`), op ids (`#2:`), value references (`%3`),
quoted op names (`"core.add"`), scalar/tensor/shape/quant types, attribute
keys, numbers, and strings.

## VS Code

1. Install the **TextMate Languages** / any extension host that accepts raw
   `.tmLanguage.json` grammars (e.g. "Grammar File Support"), or embed this
   file in a minimal extension:

   ```jsonc
   // package.json of a local extension
   {
     "contributes": {
       "languages": [
         { "id": "tpt-uir-text", "extensions": [".uir"] }
       ],
       "grammars": [
         {
           "language": "tpt-uir-text",
           "scopeName": "source.tpt-uir-text",
           "path": "./tpt-uir-text.tmLanguage.json"
         }
       ]
     }
   }
   ```

2. Place `tpt-uir-text.tmLanguage.json` next to it and reload.

## Other editors

Any TextMate-compatible editor (Sublime Text, TextMate, Zed) can consume the
file directly; map the scope name `source.tpt-uir-text` to your file type.

A full tree-sitter grammar remains future work — the TextMate grammar covers
the common highlighting needs today.
