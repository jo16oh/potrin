import {
  defineConfig,
  defineKeyframes,
  defineGlobalStyles,
  defineAnimationStyles,
} from "@pandacss/dev";
import { preset } from "@pandacss/preset-panda";
import pandaAnimate from "pandacss-animate";

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
      colors: {
        darken: { value: "rgb(0 0 0 / 0.05)" },
        lighten: { value: "rgb(255 255 255/ 0.05)" },
        transparent: { value: "transparent" },
        overlay: { value: "rgb(0 0 0 / 0.70)" },
      },
      radii: {},
      shadows: {},
      zIndex: {},
    },
    semanticTokens: {
      colors: {
        selected: {
          value: {
            base: "{colors.darken}",
            _dark: "{colors.lighten}",
          },
        },
        view: {
          bg: {
            value: {
              base: "#F7F7F7",
              _dark: "#242424",
            },
          },
          "bg-selected": {
            value: {
              base: "color-mix(in srgb, #F7F7F7, {colors.darken})",
              _dark: "color-mix(in srgb, #242424, {colors.lighten})",
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
          "bg-selected": {
            value: {
              base: "color-mix(in srgb, #FFFFFF, {colors.darken})",
              _dark: "color-mix(in srgb, #333333, {colors.lighten})",
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
          "bg-selected": {
            value: {
              base: "color-mix(in srgb, #FFFFFF, {colors.darken})",
              _dark: "color-mix(in srgb, #333333, {colors.lighten})",
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
              base: "#C8C3B9",
              _dark: "#525252",
            },
          },
          "bg-selected": {
            value: {
              base: "color-mix(in srgb, #C8C3B9, {colors.darken})",
              _dark: "color-mix(in srgb, #525252, {colors.lighten})",
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
      zIndex: {
        global: {
          float: { value: "0" },
          sidebar: { value: "10" },
          overlay: { value: "100" },
          titlebar: { value: "9999" },
        },
        local: {
          header: { value: "10" },
          scrollbar: { value: "20" },
        },
      },
    },
    extend: {
      keyframes: defineKeyframes({
        sidebarSlideIn: {
          from: {
            transform: "translateX(-110%)",
          },
          to: {
            transform: "translateX(0%)",
          },
        },
        sidebarSlideOut: {
          from: {
            transform: "translateX(0%)",
          },
          to: {
            transform: "translateX(-110%)",
          },
        },
      }),
      animationStyles: defineAnimationStyles({
        "sidebar-slide-in": {
          value: {
            animationFillMode: "forwards",
            animationDuration: "fast",
            animationName: "sidebarSlideIn",
          },
        },
        "sidebar-slide-out": {
          value: {
            animationFillMode: "forwards",
            animationDuration: "fast",
            animationName: "sidebarSlideOut",
          },
        },
      }),
    },
  },
  themes: {
    // define color schemes
    // https://panda-css.com/docs/guides/multiple-themes#multi-themes
  },

  presets: [pandaAnimate],

  // The output directory for your css system
  outdir: "styled-system",

  globalCss: defineGlobalStyles({
    "html, body": {
      overscrollBehavior: "none",
    },
    "body, body *": {
      userSelect: "none",
    },
  }),

  strictTokens: true,
  strictPropertyValues: true,
});
