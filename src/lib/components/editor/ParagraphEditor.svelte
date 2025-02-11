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

  let focus = $state(false);

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
          class: css({ _focus: { ring: "none" } }),
        },
      },
      onTransaction: () => {
        // force re-render
        editor = editor;
      },
      onBlur: () => {
        focus = false;
        setTimeout(() => {
          editor?.destroy();
        }, 0);

        if (focusPosition.id === paragraph.id) {
          focusPosition.id = null;
          focusPosition.position = null;
        }
      },
      onDestroy: () => {
        if (editor) {
          paragraph.doc = editor.getJSON();
          paragraph.save();
          editor = undefined;
        }
      },
      onFocus: () => {
        console.log("onFocus");
        focus = true;
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
    if (editor && !focus) {
      editor?.destroy();
    }
  }}
></div>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class={css(containerStyle, editorStyleVariants[editorStyleVariant])}
  style:display={editor ? "none" : "block"}
  onmouseenter={() => {
    console.log("mouseenter");
    createEditor(paragraph, null);
  }}
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
        wordBreak: "break-word",
        color: "card.text",
        minHeight: "[1.5rem]",
      },
    }),
  };
</script>
