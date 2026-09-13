# Dataset CSV Contract

Polarbear Vocab imports local CSV files directly into a selected dataset. The desktop app validates the file, shows a preview, and only enables import when every row is valid. The complete import is committed atomically, so a failed import does not leave partial vocabulary or dataset membership behind.

## Columns

| Column | Required | Meaning |
| --- | --- | --- |
| `sense_uid` | No | Stable sense identifier. Generated as `en:{lemma}:{pos}:1` when omitted. |
| `lemma` | Yes | English answer shown in the quiz. |
| `pos` | No | Part of speech. Defaults to `unknown`. |
| `quiz_prompt_zh` | Yes | Unambiguous Chinese question prompt. |
| `gloss_zh` | No | Chinese gloss shown after answering. Defaults to the quiz prompt. |
| `ipa_us` | No | US IPA. |
| `ipa_uk` | No | UK IPA. |
| `example_en` | No | Short English example sentence. |
| `example_zh` | No | Chinese example translation. |

Files must be no larger than 10 MB and may contain at most 50,000 vocabulary rows. `lemma` is limited to 100 characters and `quiz_prompt_zh` to 300 characters. A file must not repeat a `sense_uid`.

Use [`data/examples/dataset-template.csv`](../data/examples/dataset-template.csv) as a starting point. Keep Chinese prompts specific enough that only one of four English choices is valid.

Imports upsert a sense with the same stable identifier and add it to the selected dataset. Because a sense may belong to multiple datasets, changing shared content affects every dataset containing that `sense_uid`.
