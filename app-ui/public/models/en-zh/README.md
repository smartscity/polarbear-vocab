# Bundled English-Chinese translation model

These compressed model files are bundled with Polarbear Vocab so Listening
Practice can translate English articles without a network connection.

- Model family: Mozilla Firefox Translations / Bergamot
- Source: `https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/models/en-zh/llmaat_finetune10M_qe8_f2_ByQcSxGXQRqGi-UTxYE43g/exported/`
- License: Mozilla Public License 2.0 (`MPL-2.0`)
- License reference: `https://github.com/mozilla/translations/blob/main/LICENSE`

The files remain gzip-compressed in the application bundle and are
decompressed locally only when translation is requested. Do not replace a
file without updating `scripts/verify-translation-model.mjs` and the model
integrity metadata in `articleTranslation.ts`.
