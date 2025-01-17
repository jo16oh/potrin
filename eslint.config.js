import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import prettier from "eslint-config-prettier";
import globals from "globals";
import sveltePlugin from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";

// @ts-expect-error no type declaration
import panda from "@pandacss/eslint-plugin";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...sveltePlugin.configs["flat/recommended"],
  prettier,
  ...svelte.configs["flat/prettier"],
  {
    files: ["**/*.svelte", "*.svelte"],
    plugins: {
      "@pandacss": panda,
    },
    rules: {
      ...panda.configs.recommended.rules,
    },
    settings: {
      svelte: {
        ignoreWarnings: [
          "@typescript-eslint/no-unsafe-assignment",
          "@typescript-eslint/no-unsafe-member-access",
          "@typescript-eslint/no-unsafe-call",
        ],
      },
    },
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: {
          ts: "@typescript-eslint/parser",
        },
        extraFileExtensions: [".svelte"],
        svelteFeatures: {
          experimentalGenerics: true,
        },
      },
    },
  },
  {
    files: ["**/*.svelte.ts", "*.svelte.ts", "**/*.svelte.js", "*.svelte.js"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    ignores: [
      "build/",
      ".svelte-kit/",
      "src/generated/*",
      "**/*.test.ts",
      "**/*.test.js",
      "**/*.test.svelte.ts",
      "**/*.test.svelte.js",
      "styled-system",
    ],
  },
);
