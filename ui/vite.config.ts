/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // Relative URLs so `dist/` works from file://, IPFS, and GitHub Pages subpaths.
  base: "./",
  define: {
    global: "globalThis",
  },
  test: {
    environment: "node",
    include: ["test/**/*.test.ts"],
  },
});
