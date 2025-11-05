import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 5181,  // Tauri와 동일한 포트
    strictPort: true,  // 정확히 이 포트 사용
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
