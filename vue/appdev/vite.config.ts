import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueDevTools from "vite-plugin-vue-devtools";
import buildWasm from "./vite-plugin-build-wasm";

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), vueDevTools(), buildWasm()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
