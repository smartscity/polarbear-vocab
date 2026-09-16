# Polarbear Vocab User Guide

## Run

macOS requires Node.js 20.19+, pnpm 11, Rust 1.85+, and Xcode Command Line Tools.
Run these commands from the project root:

```bash
pnpm install
pnpm tauri dev
```

If pnpm is missing:

```bash
corepack enable
corepack prepare pnpm@11.19.0 --activate
```

## Build

```bash
pnpm tauri build
```

Artifacts are written to `target/release/bundle/`.

To locally bundle the six supplied full datasets (primary, junior/high school, CET-4, CET-6, IELTS):

```bash
POLARBEAR_PRELOADED_DATASETS_DIR=/Users/liyunlong/Downloads/Polarbear_Vocab_Full_Dataset_Builder/polarbear-datasets pnpm tauri build --bundles app,dmg --no-sign
```

Without this variable, the app includes only the repository's starter entries. Redistribution rights for the full CSVs are not yet verified, so do not publish this local build.
When opened, a new local build adds missing preloaded entries to an existing app data directory without deleting learning history or user-created datasets. No manual database deletion is needed.

## Use

- Lexicon: search a word, play its pronunciation, view meaning, examples and learning statistics, then practice it or add it to My Vocabulary.
- Home/Datasets: choose Unseen, Mistakes, or All; choose 10/20/50 words; start. The summary can immediately practice mistakes.
- Datasets: drag the handle to reorder datasets on macOS or iPhone; the arrow keys also work. Import or update CSV with Add only, Update existing, or Replace dataset. Export, rename, and delete are under Manage. Required columns: `lemma`, `quiz_prompt_zh`; template: [`data/examples/dataset-template.csv`](../data/examples/dataset-template.csv).
- Listening: import UTF-8 `.txt`/`.md`, choose a voice and 0.5×/1×/1.5×/2×. Select a word in the article to look it up and add it to My Vocabulary.
- Mistakes: select a filter and choose **Practice this set**.
- Settings: choose language, appearance, voice, rate, and export/import backup.

Study keys: `1`–`4` answer, `Space` next, `R` speak word, `S` speak example, `Esc` exit.

## Backup

Settings → Data → **Export Backup**. Restore with **Import Backup**. Restore automatically backs up current data first. The app remains offline and has no account or cloud sync.

## Startup issues

- Missing pnpm: run the Corepack commands above.
- Rust/macOS linker error: run `xcode-select --install`.
- Port 1420 in use: stop the earlier Vite/Tauri process and retry.
