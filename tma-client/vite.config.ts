import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "path";

const target = process.env.TARGET ?? "main_bot_mini_app";

var input = {
  main_bot_mini_app: resolve(__dirname, "src/pages/main_bot_mini_app.html"),
};

if (target === "mini_app") {
  input = {
    //@ts-ignore
    mini_app: resolve(__dirname, "src/pages/mini_app.html"),
  };
}

export default defineConfig({
  resolve: {
    alias: {
      $tgui: resolve(__dirname, "../../telegram-ui/src/lib/components/blocks/list.svelte"),
    },
  },
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
