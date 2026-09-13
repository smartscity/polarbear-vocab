import { defineConfig } from "@playwright/test";
import path from "node:path";

const repositoryRoot = process.cwd();

export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  expect: {
    toHaveScreenshot: { maxDiffPixelRatio: 0.01 },
  },
  snapshotPathTemplate: path.join(repositoryRoot, "docs/ui/screenshot-baselines/{testFilePath}/{arg}{ext}"),
  use: {
    baseURL: "http://127.0.0.1:6006",
    colorScheme: "light",
    reducedMotion: "reduce",
  },
  webServer: {
    command: "app-ui/node_modules/.bin/storybook dev -p 6006 --ci --no-open --disable-telemetry --no-version-updates -c app-ui/.storybook",
    cwd: repositoryRoot,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    url: "http://127.0.0.1:6006",
  },
});
