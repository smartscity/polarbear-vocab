# Polarbear Vocab User Guide

## Run

macOS requires Node.js 20.19+, pnpm 11, Rust 1.85+, and Xcode Command Line Tools.

```bash
cd /Users/liyunlong/git/smartscity/VocabProbe
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

## Use

- Home: select a dataset, review statistics, and choose **Continue**.
- Datasets: create a name, select it, and import CSV. Required columns are `lemma` and `quiz_prompt_zh`; use [`data/examples/dataset-template.csv`](../data/examples/dataset-template.csv).
- Listening: import a UTF-8 `.txt`/`.md` article, choose a gender or American, British, Hong Kong, Indian, or Japanese English, then play, pause, or stop at 0.5×/1×/1.5×/2×.
- Mistakes: select a filter and choose **Practice this set**.
- Settings: choose UI language, appearance, local voice style, and speech rate.

Study keys: `1`–`4` answer, `Space` next, `R` speak word, `S` speak example, `Esc` exit.

## Data

Data is stored under `~/Library/Application Support/com.polarbear.vocab/`:

- `content.db`: datasets and vocabulary content.
- `user.db`: answer history, statistics, sessions, imported articles, and settings.

The app works offline and has no remote import, account, or cloud sync. Back up both databases together.

## Startup issues

- Missing pnpm: run the Corepack commands above.
- Rust/macOS linker error: run `xcode-select --install`.
- Port 1420 in use: stop the earlier Vite/Tauri process and retry.
