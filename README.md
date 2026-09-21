# Polarbear Vocab

Polarbear Vocab is an offline-first desktop English vocabulary learning app. It records answer history and lets learners create useful study collections without schedules, streaks, or learning debt.

The macOS-first app uses a Tauri 2 shell, React/TypeScript UI, and reusable Rust core. The knowledge-base module is named **Polarbear Lexicon**.

Current capabilities include preloaded and user-named datasets, My Vocabulary dataset snapshots, transactional CSV import, four-choice quizzes, correct and mistake collections, 30-day statistics, built-in spoken-English listening packs, imported article listening with native offline speech, macOS menu-bar Services, adaptive compact/medium/wide layouts, light and dark themes, and immediate English/Simplified Chinese UI switching. It deliberately has no scheduler and no remote dataset source.

## Architecture

- `app-ui`: React and TypeScript user interface.
- `src-tauri`: Tauri command adapter and desktop shell.
- `crates/domain`: Platform-independent domain types and invariants.
- `crates/application`: Use-case orchestration and infrastructure ports.
- `crates/*-engine`: Quiz, collection, and statistics capabilities.
- `crates/lexicon`: Polarbear Lexicon read model.
- `crates/dataset-import`: Local CSV validation and import planning.
- `crates/storage-sqlite`: SQLite adapters for content and user data.
- `crates/speech`: Offline speech abstraction.
- `tools/lexicon-builder`: Reproducible preloaded content database builder.

Dependencies point inward: adapters and infrastructure depend on application, and application depends on domain. React never accesses SQLite directly.

## Development

Prerequisites: Node.js 20.19 or newer, pnpm, the Rust toolchain, and the [Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
pnpm install
pnpm tauri dev
```

Required change checks:

```bash
pnpm run typecheck
pnpm test
```

UI development and the complete screenshot/accessibility gate:

```bash
pnpm --filter @polarbear/vocab-ui storybook
pnpm test:visual
```

See the [English user guide](docs/user-guide.md) or [中文用户手册](docs/zh-CN/user-guide.md) for running and using the app. See [`trd.md`](trd.md) for the consolidated technical design and [`docs/architecture.md`](docs/architecture.md) for architecture and naming decisions.
See [`docs/dataset-csv.md`](docs/dataset-csv.md) for the CSV contract and import behavior.
