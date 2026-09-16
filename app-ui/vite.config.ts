import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig, type Plugin } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [tailwindcss(), react(), bergamotRuntimeAssets()],
  clearScreen: false,

  server: {
    host: host || false,
    port: 1420,
    strictPort: true,

    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,

    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  worker: {
    rollupOptions: { output: { entryFileNames: "assets/[name].js" } },
  },
});

function bergamotRuntimeAssets(): Plugin {
  const require = createRequire(import.meta.url);
  const translator = require.resolve("@browsermt/bergamot-translator/translator.js");
  const directory = join(dirname(translator), "worker");
  const files = ["bergamot-translator-worker.js", "bergamot-translator-worker.wasm"];
  return {
    apply: "build",
    generateBundle() {
      for (const file of files) {
        this.emitFile({
          fileName: `assets/${file}`,
          source: readFileSync(join(directory, file)),
          type: "asset",
        });
      }
    },
    name: "bergamot-runtime-assets",
  };
}
