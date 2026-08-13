import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { createReadStream, existsSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL(".", import.meta.url));
const pagesBase = process.env.GITHUB_PAGES === "1" ? "/EnvBox/" : "/";

export default defineConfig({
  root,
  base: pagesBase,
  plugins: [
    react(),
    {
      name: "envbox-download",
      configureServer(server) {
        server.middlewares.use("/downloads/EnvBox.exe", (_request, response) => {
          const executable = resolve(root, "..", "EnvBox.exe");
          if (!existsSync(executable)) {
            response.statusCode = 404;
            response.end("EnvBox.exe is not available");
            return;
          }
          response.setHeader("Content-Type", "application/vnd.microsoft.portable-executable");
          response.setHeader("Content-Disposition", "attachment; filename=EnvBox.exe");
          createReadStream(executable).pipe(response);
        });
      },
    },
  ],
  publicDir: false,
  server: {
    host: "127.0.0.1",
    port: 4174,
    strictPort: true,
    fs: {
      allow: [resolve(root, "..")],
    },
  },
  preview: {
    host: "127.0.0.1",
    port: 4174,
    strictPort: true,
  },
  css: {
    postcss: {
      plugins: [],
    },
  },
  build: {
    outDir: resolve(root, "release"),
    emptyOutDir: false,
    target: "es2022",
  },
});
