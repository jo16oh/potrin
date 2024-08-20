// @ts-check
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginSvelte from "eslint-plugin-svelte";
import globals from "globals";

export default tseslint.config({
  ignores: ["src/generated/**"],
  languageOptions: {
    globals: {
      ...globals.browser,
      ...globals.node,
    },
  },
  extends: [
    eslint.configs.recommended,
    eslintConfigPrettier,
    ...tseslint.configs.strict,
    ...eslintPluginSvelte.configs["flat/prettier"],
  ],
});
