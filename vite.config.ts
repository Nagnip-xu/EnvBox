import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望固定端口，且在 tauri dev 时不清屏
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});