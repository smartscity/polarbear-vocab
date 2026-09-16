# Polarbear Vocab Technical Requirements & Design

## 1. Scope

Polarbear Vocab is a desktop-first, offline vocabulary app. The package and repository name is `polarbear-vocab`; the product name is **Polarbear Vocab** and the knowledge module is **Polarbear Lexicon**.

This document consolidates the implemented design through v0.17:

| Version | Delivered scope |
| --- | --- |
| v0.4 | Tauri + Rust foundation, `content.db` / `user.db`, quiz, correct/mistake collections, statistics, offline TTS |
| v0.8 | User-named datasets, local CSV import, preloaded datasets; no catalog, GitHub, or remote import |
| v0.9 | English and Simplified Chinese UI with live switching in Settings |
| v0.10 | Adaptive compact/medium/wide UI, light/dark/system themes, reusable design system, visual and accessibility gates |
| v0.11 | Local TXT/Markdown article library and MP3-style offline listening controls |
| v0.12 | Polarbear Lexicon search, word detail, word statistics, and My Vocabulary |
| v0.13 | User-sized study sessions, persisted checkpoints, and completion summary |
| v0.14 | Single-file ZIP backup/restore with manifest and automatic pre-restore backup |
| v0.15 | Dataset update strategies, CSV export, rename, and delete lifecycle |
| v0.16 | Listening text selection to Lexicon and My Vocabulary |
| v0.17 | iPhone document-picker, background speech, interruption, checkpoint, and data-transfer behavior |

Non-goals: scheduler, spaced repetition, due dates, streaks, daily targets, accounts, cloud sync, remote dataset sources, and online TTS.

## 2. Architecture

```text
React UI
   ↓ typed Tauri commands
Tauri adapter (`src-tauri`)
   ↓ application ports
Rust application + domain + engines
   ↓ adapters
SQLite (`content.db`, `user.db`) + native offline speech
```

Dependencies point inward. The UI does not access SQLite, the filesystem, or native speech directly.

| Module | Responsibility |
| --- | --- |
| `app-ui` | React UI, localization, theme, adaptive layout, Storybook |
| `src-tauri` | Desktop shell and command boundary |
| `crates/domain` | Shared types, invariants, DTO contracts |
| `crates/application` | Use-case services and infrastructure ports |
| `crates/quiz-engine` | Four-option question validation and answer rules |
| `crates/collection-engine` | Unseen, explored, correct, dataset, and mistake collections |
| `crates/statistics-engine` | History-derived totals, mistake buckets, daily activity |
| `crates/lexicon` | Read model for senses, pronunciation, and examples |
| `crates/dataset-import` | CSV validation and deterministic import plan |
| `crates/storage-sqlite` | Transactions, schema migration, repositories, read models |
| `crates/speech` | Offline speech abstraction |
| `tools/lexicon-builder` | Reproducible preloaded `content.db` builder |

## 3. Data design

### `content.db`

Stores preloaded and imported datasets, dataset membership, words, senses, IPA, translations, and examples. User imports are transactional: any invalid row aborts the full import.

The builder can additionally read six supplied full CSV datasets from `POLARBEAR_PRELOADED_DATASETS_DIR` during a local build. It validates headers and identifiers, preserves every dataset membership, chooses one deterministic richer record for shared `sense_uid`s, and generates three distinct-lemma quiz distractors per sense. The CSVs are not checked into this repository while redistribution permission remains unverified; the source record marks locally built content as not cleared for redistribution.

On startup, the app hashes the bundled seed and applies each new seed once to the writable `content.db`. The upgrade backs up the existing database, transactionally inserts missing senses and preloaded memberships, and preserves user-created datasets and existing stable UIDs. Failed upgrades restore the backup. Lexicon search queries all senses in the merged content database, independent of the active dataset.

### `user.db`

Stores append-only answer history, study sessions, My Vocabulary, derived progress/statistics, imported articles, UI language, theme, and speech preferences. Correct and mistake collections are derived from history; a later correct answer does not erase an earlier mistake.

Stable identifiers use `dataset_id` and `sense_uid`. Current content schema is v3 and user schema is v5. Migrations convert legacy `dataset_uid` columns, add dataset update timestamps and persisted session new-word counts, and make a consistent SQLite backup before changing `user.db`.

## 4. Dataset import

The user creates a locally named dataset and imports a UTF-8 CSV file. Required columns are:

- `lemma`
- `quiz_prompt_zh`

Optional columns include stable UID, IPA, gloss, part of speech, and examples. Preview reports row-level issues and duplicate identifiers. Missing UIDs are generated deterministically, including for non-ASCII lemmas. The final write runs in one SQLite transaction.

Dataset detail shows source, update time, word count, Start, Import / Update, Export CSV, Rename, and Delete. Import behavior is explicit:

Dataset list order is user-controlled through a pointer or touch drag handle, with arrow-key support. The ordered dataset IDs are stored as a user preference in `user.db`, so bundled content upgrades do not reset the order.

- **Add only** adds membership and new senses without changing existing senses.
- **Update existing** adds new entries and overwrites matching stable `sense_uid` content.
- **Replace dataset** replaces membership with the imported file and updates matching content.

CSV export writes the complete re-importable local dataset schema. No remote catalog or source exists.

There is no GitHub import, online catalog, or network request.

## 5. Learning flow

1. The user selects Unseen, Mistakes, or All and chooses 10, 20, or 50 words.
2. Resolve that collection from the selected dataset and answer history, then apply the session limit.
3. Build a question with one Chinese prompt and exactly four unique English options.
4. Validate the selected option against the active question.
5. Write one answer-history event and checkpoint transactionally; stale or repeated submissions cannot create duplicates.
6. Return correctness, whether the word is new, the correct sense, pronunciation, gloss, and example.
7. Show answered/correct/wrong/new totals and allow immediate practice of this session's mistakes.
8. Recompute home and mistake views from recorded history.

Study keyboard controls are `1`–`4`, `Space`, `R`, `S`, and `Esc`. Speech uses the operating system voice and never calls a cloud service.

## 6. UI design

The design system lives under `app-ui/src/design-system` and contains tokens, typography, motion, primitives, and reusable screen components. Radix primitives provide accessible select and dialog behavior; feature CSS stays close to each screen.

Responsive modes:

| Width | Navigation | Layout |
| --- | --- | --- |
| `< 640px` | Bottom navigation | Compact, safe-area padding, minimum 44×44 px targets |
| `640–1023px` | Top navigation | Medium centered content |
| `≥ 1024px` | Left rail | Wide desktop workspace |

Study hides global navigation so the question position remains stable. The UI supports pointer and touch input, visible keyboard focus, reduced motion, 200% text zoom, and no horizontal overflow.

Themes are `system`, `light`, and `dark`. The initial system theme is applied before React paints; the saved choice is then loaded from `user.db`. UI language is `system`, `en`, or `zh-CN`; switching language does not translate dataset content. Speech settings persist the `en-US` / `en-GB` pronunciation, voice style, and one of the four supported rates.

## 7. Article listening

The user imports a UTF-8 `.txt` or `.md` file. The Tauri adapter reads it locally, the application layer validates a 1–160 character title and 1–100,000 character body, and `user.db` stores it in the `article` table. No document content leaves the device.

The Listening screen provides a local article library, readable text, and play, pause, resume, stop, and delete actions. Playback uses `AVSpeechSynthesizer` with exact rates 0.5×, 1×, 1.5×, and 2×. Voice choices are male, female, American English (`en-US`), British English (`en-GB`), Hong Kong English (`en-HK`), Indian English (`en-IN`), and Japanese English (`ja-JP`); unavailable voices fall back to a system voice.

Selecting an English word in an article performs a local Lexicon lookup. The result shows lemma, IPA, and Chinese gloss and can be added to My Vocabulary, which is a practiceable collection.

## 8. Lexicon search

Search is a prefix lookup over local `word` and `sense` records; exact matches rank first. SQL wildcard characters are escaped. Each result includes IPA, part of speech, Chinese gloss, primary example, dataset memberships, answered/correct/wrong counts, My Vocabulary state, and direct Practice.

## 9. Backup and restore

Settings exports one `.polarbear-vocab-backup` ZIP containing exactly:

```text
manifest.json
content.db
user.db
```

The manifest contains `schema_version`, `app_version`, and `created_at`. Export uses SQLite's online backup API so WAL data is included consistently. Import rejects unknown entries, oversized files, invalid manifests, and incompatible database schemas. Before restore, the current databases are automatically exported beside `user.db`; both restored databases remain behind the repository boundary.

## 10. iPhone native behavior

| Situation | Behavior |
| --- | --- |
| Import CSV/TXT/Markdown/backup | Tauri dialog opens the iOS Document Picker; files may be opened in place |
| Call, Siri, or route interruption | AVFoundation owns interruption and route handling; if speech does not resume, the user can Stop and Play again. Automatic interruption recovery has not been verified on a physical iPhone |
| Screen lock or background | `AVAudioSessionCategoryPlayback` plus `UIBackgroundModes=audio` makes Listening speech eligible to continue; physical-device verification remains required |
| Quiz background/termination | Every accepted answer updates `study_session` and `session_item` in the same transaction; Home offers Resume for the newest incomplete session |
| Desktop/mobile transfer | Export on one device and import the same backup file on the other; there is no cloud sync |

The same typed commands and databases are used on desktop and iPhone. Mobile-specific behavior is native integration, not a separate product model.

## 11. Verification and release

Required local gate:

```bash
pnpm run check
pnpm run build
pnpm --filter @polarbear/vocab-ui run storybook:build
pnpm run test:visual
```

Coverage includes Rust domain/application/engine/storage tests, frontend locale/layout tests, ten screens at seven viewport sizes, dark appearance, axe WCAG checks, compact touch targets, focus visibility, 200% text zoom, and screenshot baselines under `docs/ui/screenshot-baselines`.

A `vMAJOR.MINOR.PATCH` tag is the release source of truth. CI validates the tag, injects the version, runs all gates, then builds platform artifacts. Release artifact paths are rooted at `target/release/bundle/`.

The current tag workflow produces an unsigned macOS universal DMG and `.app` ZIP. Apple certificates/notarization are intentionally disabled with `--no-sign`; macOS may show a Gatekeeper warning for downloaded artifacts. iPhone IPA release is paused because device distribution requires Apple signing credentials.

## 12. Run and build

```bash
pnpm install
pnpm tauri dev
```

Build an installer with `pnpm tauri build`. See [中文用户手册](docs/zh-CN/user-guide.md) or [English User Guide](docs/user-guide.md).
