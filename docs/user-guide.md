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

Every build includes the six supplied full datasets (primary, junior/high school, CET-4, CET-6, IELTS). Build and run on a USB-connected iPhone with:

```bash
pnpm tauri ios run
pnpm tauri ios build --open
```

Both commands rebuild and package the same full `content.db` used by macOS. A new build adds missing preloaded entries to an existing app data directory without deleting learning history or user-created datasets. No database deletion is needed. Audit the supplied CSVs' upstream redistribution rights before publishing them.

## Use

- Lexicon: search a word, play its pronunciation, view meaning, examples and learning statistics, then practice it or add it to My Vocabulary.
- Home/Datasets: choose Unseen, Mistakes, or All; choose 10/20/50 words; start. The summary can immediately practice mistakes.
- Datasets: drag the handle to reorder datasets on macOS or iPhone; the arrow keys also work. Import or update CSV with Add only, Update existing, or Replace dataset. Export, rename, and delete are under Manage. Required columns: `lemma`, `quiz_prompt_zh`; template: [`data/examples/dataset-template.csv`](../data/examples/dataset-template.csv).
- Listening: import UTF-8 `.txt`/`.md`, choose a voice and 0.5×/1×/1.5×/2×. Markdown is rendered as a document. Translate English to Chinese, then copy English, Chinese, or both. Wide screens place both languages side by side; iPhone stacks them. The first translation downloads and caches the offline model. Select an English word to look it up and add it to My Vocabulary.
- Mistakes: select a filter and choose **Practice this set**.
- Settings: choose language, appearance, voice, rate, and export/import backup.

Study keys: `1`–`4` answer, `Space` next, `R` speak word, `S` speak example, `Esc` exit.

## Backup

Settings → Data → **Export Backup**. Restore with **Import Backup**. Restore automatically backs up current data first. The app remains offline and has no account or cloud sync.

## Startup issues

- Missing pnpm: run the Corepack commands above.
- Rust/macOS linker error: run `xcode-select --install`.
- Port 1420 in use: stop the earlier Vite/Tauri process and retry.
- iPhone import: choose the CSV/TXT/Markdown file through the iOS document picker; files under **On My iPhone** are copied into the app sandbox automatically.
