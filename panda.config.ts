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
      radii: {},
      shadows: {},
    },
    semanticTokens: {
      colors: {
        selected: {
          value: {
            base: "rgb(0 0 0 / 0.05)",
            _dark: "rgb(255 255 255 / 0.05)",
          },
        },
        view: {
          bg: {
            value: {
              base: "#F7F7F7",
              _dark: "#242424",
            },
          },
          text: {
            value: {
              base: "#0C0A09",
              _dark: "#F8F8F8",
            },
          },
          "text-muted": {
            value: {
              base: "#78716C",
              _dark: "C7C7C7",
            },
          },
        },
        card: {
          bg: {
            value: {
              base: "#FFFFFF",
              _dark: "#333333",
            },
          },
          text: {
            value: {
              base: "#0C0A09",
              _dark: "#F8F8F8",
            },
          },
          "text-muted": {
            value: {
              base: "#78716C",
              _dark: "C7C7C7",
            },
          },
        },
        button: {
          bg: {
            value: {
              base: "#FFFFFF",
              _dark: "#333333",
            },
          },
          text: {
            value: {
              base: "#0C0A09",
              _dark: "#F8F8F8",
            },
          },
          "text-muted": {
            value: {
              base: "#78716C",
              _dark: "C7C7C7",
            },
          },
        },
        accent: {
          bg: {
            value: {
              base: "#A39E93",
              _dark: "#525252",
            },
          },
          text: {
            value: {
              base: "#FEFEFE",
              _dark: "#F9F9F9",
            },
          },
          "text-muted": {
            value: {
              base: "#78716C",
              _dark: "9D948D",
            },
          },
        },
      },
      radii: {
        md: {
          value: "0.5rem",
        },
        lg: {
          value: "1rem",
        },
        circle: {
          value: "50%",
        },
      },
      shadows: {
        sm: {
          value: {
            offsetX: 0,
            offsetY: 1,
            blur: 1,
            spread: 0,
            color: "rgb(0 0 0 / 0.25)",
          },
        },
        md: {
          DEFAULT: {
            value: {
              offsetX: 0,
              offsetY: 4,
              blur: 4,
              spread: 0,
              color: "rgb(0 0 0 / 0.25)",
            },
          },
          around: {
            value: {
              offsetX: 0,
              offsetY: 0,
              blur: 4,
              spread: 0,
              color: "rgb(0 0 0 / 0.25)",
            },
          },
          inner: {
            value: {
              offsetX: 0,
              offsetY: 0,
              blur: 3,
              spread: 0,
              color: "rgb(0 0 0 / 0.60)",
              inset: true,
            },
          },
        },
        lg: {
          value: {
            offsetX: 0,
            offsetY: 2,
            blur: 8,
            spread: 0,
            color: "rgb(0 0 0 / 0.40)",
          },
        },
      },
      //@ts-expect-error define tokens which doesn't have default values
      dropShadows: {
        md: { value: "drop-shadow(0px 4px 4px rgb(0 0 0 / 0.25))" },
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
