// @ts-check

import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginSvelte from "eslint-plugin-svelte";

export default [
  eslint.configs.recommended,
  eslintConfigPrettier,
  ...tseslint.configs.strict,
  ...eslintPluginSvelte.configs["flat/prettier"],
];
