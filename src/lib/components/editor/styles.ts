import { cva } from "styled-system/css/cva";

export const outlineEditorStyle = cva({
  base: {
    position: "relative",
    w: "full",
    h: "fit",
  },
  variants: {
    style: {
      cardsViewTitle: {
        minHeight: "[3rem]",
        color: "view.text",
        wordBreak: "break-word",
        "& p": {
          color: "view.text",
          fontSize: "[2rem]",
          fontWeight: "semibold",
        },
      },
    },
  },
});

export const paragraphEditorStyle = cva({
  base: {
    minH: "[4.225rem]",
    "& p": {
      minH: "[1.6rem]",
      color: "card.text",
      userSelect: "text",
    },
  },
  variants: {
    style: {
      card: {
        position: "relative",
        rounded: "lg",
        w: "full",
        h: "fit",
        py: "[1.3125rem]",
        px: "[1.75rem]",
        bg: "card.bg",
        shadow: "sm",
      },
    },
  },
});
