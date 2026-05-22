import { defineConfig } from "electron-vite";
import vue from "@vitejs/plugin-vue";
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
    plugins: [vue()],
    resolve: {
      alias: {
        "@": resolve(import.meta.dirname, "src/renderer"),
      },
    },
    build: {
      rollupOptions: {
        input: resolve(import.meta.dirname, "src/renderer/index.html"),
      },
    },
  },
});
