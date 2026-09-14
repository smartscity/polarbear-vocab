# Polarbear Vocab Technical Requirements & Design

## 1. Scope

Polarbear Vocab is a desktop-first, offline vocabulary app. The package and repository name is `polarbear-vocab`; the product name is **Polarbear Vocab** and the knowledge module is **Polarbear Lexicon**.

This document consolidates the implemented design through v0.10:

| Version | Delivered scope |
| --- | --- |
| v0.4 | Tauri + Rust foundation, `content.db` / `user.db`, quiz, correct/mistake collections, statistics, offline TTS |
| v0.8 | User-named datasets, local CSV import, preloaded datasets; no catalog, GitHub, or remote import |
| v0.9 | English and Simplified Chinese UI with live switching in Settings |
| v0.10 | Adaptive compact/medium/wide UI, light/dark/system themes, reusable design system, visual and accessibility gates |

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

### `user.db`

Stores append-only answer history, study sessions, derived progress/statistics, UI language, and theme. Correct and mistake collections are derived from history; a later correct answer does not erase an earlier mistake.

Stable identifiers use `dataset_id` and `sense_uid`. Schema migration converts the legacy `dataset_uid` column without losing data.

## 4. Dataset import

The user creates a locally named dataset and imports a UTF-8 CSV file. Required columns are:

- `lemma`
- `quiz_prompt_zh`

Optional columns include stable UID, IPA, gloss, part of speech, and examples. Preview reports row-level issues and duplicate identifiers. Missing UIDs are generated deterministically, including for non-ASCII lemmas. The final write runs in one SQLite transaction.

There is no GitHub import, online catalog, or network request.

## 5. Learning flow

1. Resolve a collection from the selected dataset and answer history.
2. Build a question with one Chinese prompt and exactly four unique English options.
3. Validate the selected option against the active question.
4. Write one answer-history event transactionally; stale or repeated submissions cannot create duplicates.
5. Return correctness, the correct sense, pronunciation, gloss, and example.
6. Recompute home and mistake views from recorded history.

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

Themes are `system`, `light`, and `dark`. The initial system theme is applied before React paints; the saved choice is then loaded from `user.db`. UI language is `system`, `en`, or `zh-CN`; switching language does not translate dataset content. Speech settings persist the `en-US` / `en-GB` voice and a 50–200% rate.

## 7. Verification and release

Required local gate:

```bash
pnpm run check
pnpm run build
pnpm --filter @polarbear/vocab-ui run storybook:build
pnpm run test:visual
```

Coverage includes Rust domain/application/engine/storage tests, frontend locale/layout tests, seven screens at seven viewport sizes, dark appearance, axe WCAG checks, compact touch targets, focus visibility, 200% text zoom, and screenshot baselines under `docs/ui/screenshot-baselines`.

A `vMAJOR.MINOR.PATCH` tag is the release source of truth. CI validates the tag, injects the version, runs all gates, then builds platform artifacts. Release artifact paths are rooted at `target/release/bundle/`.

## 8. Run and build

```bash
pnpm install
pnpm tauri dev
```

Build an installer with `pnpm tauri build`. See [中文用户手册](docs/zh-CN/user-guide.md) or [English User Guide](docs/user-guide.md).
