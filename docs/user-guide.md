# Polarbear Vocab User Guide

## Run from source on macOS

Install these prerequisites first:

- Node.js 20.19 or newer.
- pnpm 11.
- Rust 1.85 or newer through `rustup`.
- Xcode Command Line Tools (`xcode-select --install`).

From the repository root, install dependencies and start the desktop app:

```bash
cd /Users/liyunlong/git/smartscity/VocabProbe
pnpm install
pnpm tauri dev
```

The first command installs the locked JavaScript dependencies. The second rebuilds the preloaded Polarbear Lexicon database, starts Vite, compiles the Rust application, and opens the Tauri desktop window. The first Rust build can take several minutes.

If `pnpm` is not installed, enable Corepack and activate the repository's pinned pnpm version:

```bash
corepack enable
corepack prepare pnpm@11.19.0 --activate
```

## Build an installable application

Run:

```bash
pnpm tauri build
```

macOS application bundles and installers are written under `target/release/bundle/`. Use `pnpm run build` only when you need to rebuild the lexicon and web interface without packaging the desktop shell.

## First use

The app opens on Home with a preloaded dataset selected. Home shows explored vocabulary, correct and wrong collections, 30-day activity, and mistake shortcuts. Select another dataset at the top of Home or open Datasets from the compact navigation.

To study unseen items, choose **Continue**. To revisit prior work, choose the explored, correct, or wrong statistic, or open a mistake threshold. Quiz questions show one Chinese prompt and exactly four English choices.

Keyboard controls during a study session:

| Key | Action |
| --- | --- |
| `1` / `2` / `3` / `4` | Choose an answer. |
| `Space` | Move to the next question after answering. |
| `R` | Speak the correct word after answering. |
| `S` | Speak the example sentence after answering. |
| `Esc` | End the current session and return Home. |

Speech is generated locally with the macOS system voice. The app does not call a cloud speech service.

## Create and import a dataset

1. Open **Datasets**.
2. Enter any dataset name and choose **Create Dataset**.
3. Select the new dataset and choose **Import CSV**.
4. Select a local `.csv` file.
5. Review the sample and validation summary.
6. Fix every reported issue, then choose **Import**.

Only `lemma` and `quiz_prompt_zh` are required. The complete format and a starter file are available in the [Dataset CSV Contract](dataset-csv.md) and [`data/examples/dataset-template.csv`](../data/examples/dataset-template.csv). Import is atomic: a failure rolls back the complete import.

Dataset detail also provides Start, Rename, and Delete. Deleting a dataset removes its membership list but preserves recorded answer history.

## Mistakes and statistics

Open **Mistakes** to filter the current dataset by all mistakes, wrong count thresholds, or the most recent result. **Practice this set** starts a quiz from the visible collection. A later correct answer does not erase earlier mistakes.

Home statistics describe actual activity. Polarbear Vocab has no due dates, daily goals, streaks, spaced-repetition schedule, or learning debt.

## Change the interface language

Open **Settings**, then choose System, English, or Simplified Chinese. The interface changes immediately and the selection is saved in `user.db`. System mode follows macOS when it uses a supported Chinese locale and otherwise falls back to English. Changing the interface language does not translate dataset content.

## Local data

On macOS, writable application data is stored under:

```text
~/Library/Application Support/com.polarbear.vocab/
```

- `content.db` contains preloaded datasets, imported datasets, vocabulary senses, pronunciations, and examples.
- `user.db` contains answer history, derived statistics, study sessions, and settings.

Back up both files before manually moving or replacing application data. The normal app workflow is fully local and does not provide a remote catalog, account, or cloud sync.

## Common startup problems

- **`pnpm: command not found`**: enable Corepack using the commands above, or install pnpm 11.
- **`node: command not found`**: install Node.js 20.19 or newer and reopen the terminal.
- **Rust linker or macOS framework error**: run `xcode-select --install`, then retry after installation finishes.
- **Port 1420 is already in use**: stop the earlier Vite or Tauri development process, then run `pnpm tauri dev` again.
- **The first launch appears slow**: the first development run compiles the Rust and Tauri dependency graph; later launches reuse the build cache.
