import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { internalIpV4 } from "internal-ip";
import { exec } from "node:child_process";
import util from "node:util";

// @ts-expect-error process is a nodejs global
const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);

// https://vitejs.dev/config/
// @ts-expect-error description
export default defineConfig(async () => ({
  plugins: [sveltekit(), typePredicates()],

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
  },
}));

function typePredicates() {
  const command = `pnpm exec type-predicates-generator -f 'src/lib/**/*.ts' -o 'src/generated/type-predicates.ts'`;
  async function generate() {
    const { stdout } = await util.promisify(exec)(command);
    console.log(stdout);
  }

  return {
    name: "type-predicates-generator",
    configureServer(server) {
      server.watcher.unwatch("src/generated/type-predicates.ts");
    },
    async buildStart() {
      await generate();
    },
    async handleHotUpdate() {
      await generate();
    },
  };
}
