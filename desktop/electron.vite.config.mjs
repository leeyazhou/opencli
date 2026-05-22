import { defineConfig } from "electron-vite";
import { resolve } from "path";

export default defineConfig({
  main: {
    entry: "src/main/index.js",
  },
  preload: {
    entry: "src/preload/index.js",
  },
  renderer: {
    root: "src/renderer",
    build: {
      rollupOptions: {
        input: resolve(import.meta.dirname, "src/renderer/index.html"),
      },
    },
  },
});
