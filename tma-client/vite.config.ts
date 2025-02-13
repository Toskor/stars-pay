import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "path";

const target = process.env.TARGET ?? "main_bot_mini_app";

var input = {
  main_bot_mini_app: "./src/pages/main_bot_mini_app.html",
};

if (target === "mini_app") {
  input = {
    mini_app: "./src/pages/mini_app.html",
  };
}

export default defineConfig({
  plugins: [
    svelte(),
    viteSingleFile({
      useRecommendedBuildConfig: true,
    }),
  ],
  build: {
    emptyOutDir: false,
    outDir: "./dist",
    rollupOptions: {
      input,
      output: {
        entryFileNames: "[name].js",
        assetFileNames: "[name].[ext]",
      },
    },
  },
});
