# Polarbear Vocab datasets

Generated CSV schema:

```
sense_uid,lemma,pos,quiz_prompt_zh,gloss_zh,ipa_us,ipa_uk,example_en,example_zh
```

Datasets:

- `polarbear-primary-pep.csv`: PEP primary grades 3–6, merged and de-duplicated.
- `polarbear-junior.csv`: junior-school vocabulary source list.
- `polarbear-high-school.csv`: high-school vocabulary source list.
- `polarbear-cet4.csv`: CET-4 learning vocabulary source list.
- `polarbear-cet6.csv`: CET-6 learning vocabulary source list.
- `polarbear-ielts.csv`: IELTS learning vocabulary set. IELTS does not publish one single official fixed mandatory word list.

The lexicon builder validates and bundles these files into the six preloaded
Datasets. A regular desktop or iOS build requires no environment variable and
does not download dataset content. `POLARBEAR_PRELOADED_DATASETS_DIR` remains an
optional local override for dataset-builder development.

## Data provenance

These checked-in CSV files were generated from source data obtained from:
https://github.com/KyleBing/english-vocabulary

Before redistributing these generated CSVs in a public/commercial App Store release,
audit the upstream dataset provenance and redistribution rights. Do not assume the
repository's availability alone establishes all downstream content rights.
