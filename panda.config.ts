import { defineConfig } from "@pandacss/dev";
import { preset } from "@pandacss/preset-panda";

export default defineConfig({
  // Whether to use css reset
  preflight: true,

  // Where to look for your css declarations
  include: ["./src/**/*.{js,ts,svelte}"],

  // Files to exclude
  exclude: [],

  conditions: {
    extend: {
      light: "[data-color-mode=light] &",
      dark: "[data-color-mode=dark] &",
    },
  },

  theme: {
    ...preset.theme,
    // @ts-expect-error exactOptionalPropertyTypes
    tokens: {
      ...preset.theme.tokens,
      colors: {},
    },
    semanticTokens: {
      colors: {
        sub: {
          base: {
            value: {
              base: "#000000",
              _dark: "#BF996A",
            },
          },
          selected: {
            value: {
              base: "#000000",
              _dark: "#BF996A",
            },
          },
        },
      },
    },
  },
  themes: {
    // define color schemes
    // https://panda-css.com/docs/guides/multiple-themes#multi-themes
  },

  // The output directory for your css system
  outdir: "styled-system",

  strictTokens: true,
  strictPropertyValues: true,
});
