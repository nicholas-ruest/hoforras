/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    host: true,
    // Allow the GitHub Codespaces / generic cloud-IDE forwarded hosts (Vite 5.4 blocks unknown
    // hosts by default). Localhost still works for native dev.
    allowedHosts: [".app.github.dev", ".github.dev", "localhost", "127.0.0.1"],
    // HMR travels over the https port-forward proxy (wss on 443).
    hmr: { clientPort: 443 },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    css: false,
  },
});
