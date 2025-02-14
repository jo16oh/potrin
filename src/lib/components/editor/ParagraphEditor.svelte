<script lang="ts">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css, type Styles } from "styled-system/css";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import { createParagraphSchema } from "./schema";
  import type { FocusPosition, EditorFocusPosition } from "./utils";
  import { watch } from "runed";

  type EditorStyleVariant = "card";

  type Props = {
    paragraph: Paragraph;
    focusPosition: FocusPosition;
    containerStyle?: Styles;
    editorStyleVariant: EditorStyleVariant;
  };

  let {
    paragraph,
    focusPosition = $bindable(),
    containerStyle,
    editorStyleVariant,
  }: Props = $props();

  const isFocused = $derived(focusPosition.id === paragraph.id);

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  watch(
    () => focusPosition,
    () => {
      if (focusPosition && focusPosition.id === paragraph.id) {
        if (editor) {
          editor.commands.focus(focusPosition.position);
        } else {
          (async () => await createEditor(paragraph, focusPosition.position))();
        }
      } else {
        editor?.commands.blur();
      }
    },
  );

  async function createEditor(paragraph: Paragraph, pos: EditorFocusPosition) {
    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions: [
        ...(await createParagraphSchema(
          paragraph,
          (pos) => (focusPosition = pos),
        )),
      ],
      editorProps: {
        attributes: {
          class: css({ ring: "none" }),
        },
      },
      onBlur: () => {
        setTimeout(() => {
          editor?.destroy();

          if (focusPosition.id === paragraph.id) {
            focusPosition = {
              id: null,
              position: null,
            };
          }
        });
      },
      onDestroy: () => {
        if (editor) {
          paragraph.doc = editor.getJSON();
          paragraph.save();
          editor = undefined;
        }
      },
      onFocus: () => {
        if (focusPosition.id !== paragraph.id) {
          focusPosition = {
            id: paragraph.id,
            position: editor!.state.selection.from,
          };
        }
      },
    });

    if (pos !== null && pos !== undefined) {
      editor.commands.focus(pos);
    }
  }

  onDestroy(() => {
    if (editor) {
      editor.destroy();
    }
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={editorElement}
  class={css(containerStyle, editorStyleVariants[editorStyleVariant])}
  style:display={editor ? "block" : "none"}
  onmouseleave={() => {
    if (editor && !isFocused) {
      editor?.destroy();
    }
  }}
></div>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class={[
    css(containerStyle, editorStyleVariants[editorStyleVariant], {
      ring: "none",
    }),
    "tiptap",
    "ProseMirror",
  ].join(" ")}
  style:display={editor ? "none" : "block"}
  onmouseenter={() => {
    createEditor(paragraph, null);
  }}
  contenteditable
>
  {#if paragraph.doc}
    {#each paragraph.doc.content ?? [] as content}
      {#if content.type === "paragraph"}
        <p>
          {#each content.content ?? [] as c}
            {#if c.type === "text"}
              {c.text}
            {/if}
          {/each}
        </p>
      {/if}
    {/each}
  {/if}
</div>

<script module>
  const editorStyleVariants = {
    card: css.raw({
      "& p": {
        color: "card.text",
        minHeight: "[1.5rem]",
      },
    }),
  };
</script>
