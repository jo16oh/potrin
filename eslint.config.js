import js from "@eslint/js";
import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import prettier from "eslint-config-prettier";
import tailwind from "eslint-plugin-tailwindcss";
import globals from "globals";
import sveltePlugin from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";

export default ts.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...tailwind.configs["flat/recommended"],
  ...sveltePlugin.configs["flat/recommended"],
  prettier,
  ...svelte.configs["flat/prettier"],
  {
    files: ["**/*.svelte", "*.svelte"],
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
    files: ["**/*.svelte.ts", "*.svelte.ts"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tseslint.parser,
      },
    },
  },
  {
    files: ["**/*.svelte.js", "*.svelte.js"],
    languageOptions: {
      parser: svelteParser,
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
    ],
  },
);
