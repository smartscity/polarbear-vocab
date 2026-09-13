# Visual Language

Polarbear Vocab uses a quiet editorial interface: warm neutral surfaces, deep evergreen primary actions, restrained borders, serif display text, and sans-serif controls. The interface must feel calm and durable rather than gamified.

All production colors, spacing, radii, shadows, and content widths come from `app-ui/src/design-system/tokens.css`. Components must use semantic tokens so the same hierarchy remains legible in light and dark themes. Focus rings, success colors, and danger colors must never rely on color alone to communicate state.

Animation is limited to 120–180 ms state transitions. Reduced-motion preferences disable nonessential movement. Cards are used only to group a real task or content region; individual metrics should not become separate decorative cards.

Storybook fixtures are deterministic and local: they must not call Tauri, SQLite, network services, or user databases. Run `pnpm test:visual` after UI changes. Intentional baseline changes require review before regeneration with Playwright's `--update-snapshots` option.
