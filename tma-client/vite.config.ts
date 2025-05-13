import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { sveltekit } from "@sveltejs/kit/vite";
import { resolve } from "path";

const target = process.env.TARGET || "main_bot_mini_app";
const isDev = process.env.NODE_ENV === "development";

var input = {};

if (target === "main_bot_mini_app") {
  input = {
    main_bot_mini_app: resolve(__dirname, "src/pages/main_bot_mini_app.html"),
  };
} else if (target === "mini_app") {
  input = {
    mini_app: resolve(__dirname, "src/pages/mini_app.html"),
  };
} else if (target === "layer") {
  input = {
    layer: resolve(__dirname, "src/pages/layer.html"),
  };
} else if (target === "blocked_app") {
  input = {
    blocked_app: resolve(__dirname, "src/pages/blocked_app.html"),
  };
}

const fileGenerationConfig = {
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
};

const devConfig = {
  plugins: [sveltekit()],
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
};

export default defineConfig(isDev ? devConfig : fileGenerationConfig);
