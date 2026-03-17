import { defineConfig } from "@rsbuild/core";
import { pluginVue } from "@rsbuild/plugin-vue";

// Docs: https://rsbuild.rs/config/
export default defineConfig({
  plugins: [pluginVue()],
  server: {
    port: 3000,
    base: "/app",
  },
  html: {
    title: "START",
  },
  source: {
    preEntry: ["./src/global.ts"],
  }
});
