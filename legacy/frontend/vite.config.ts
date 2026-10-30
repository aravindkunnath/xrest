import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wails from "@wailsio/runtime/plugins/vite";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    host: "127.0.0.1",
    port: Number(process.env.WAILS_VITE_PORT) || 9245,
    strictPort: true,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  plugins: [vue(), wails("./bindings"), tailwindcss()],
  build: {
    target: "es2022",
    cssCodeSplit: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("@codemirror") || id.includes("codemirror")) {
              return "codemirror";
            }
            if (id.includes("reka-ui") || id.includes("@tanstack")) {
              return "ui-vendor";
            }
            if (id.includes("@remixicon") || id.includes("@lucide")) {
              return "icons";
            }
            if (id.includes("@wailsio")) {
              return "wails";
            }
            if (id.includes("vue") || id.includes("pinia")) {
              return "vue-core";
            }
          }
        },
      },
    },
  },
});
