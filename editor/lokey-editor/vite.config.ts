import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import devtools from "solid-devtools/vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [devtools(), solidPlugin(), tailwindcss()],
  resolve: {
    alias: {
      "solid-js": path.resolve(
        path.dirname(fileURLToPath(import.meta.url)),
        "node_modules/solid-js",
      ),
    },
  },
  server: {
    port: 3000,
  },
  build: {
    target: "esnext",
  },
});
