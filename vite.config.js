import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { internalIpV4 } from "internal-ip";

// @ts-expect-error process is a nodejs global
const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);

// https://vitejs.dev/config/
// @ts-expect-error no overloads matches this all
export default defineConfig(async () => ({
  plugins: [sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: mobile ? "0.0.0.0" : false,
    hmr: mobile
      ? {
          protocol: "ws",
          host: await internalIpV4(),
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  test: {
    globalSetup: "./vitest.setup.ts",
    include: ["src/**/*.{test,test.svelte,spec}.{js,ts}"],
    exclude: ["src/generated/**"],
    environment: "jsdom",
    poolOptions: {
      threads: { singleThread: true },
      forks: { singleFork: true },
    },
  },
}));
