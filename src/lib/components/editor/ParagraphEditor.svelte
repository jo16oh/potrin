<script lang="ts">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css } from "styled-system/css";
  import type { Paragraph } from "$lib/models/Paragraph.svelte";
  import Collabolation from "@tiptap/extension-collaboration";
  import { ParagraphSchema } from "./schema";

  type Props = { paragraph: Paragraph };

  let { paragraph }: Props = $props();

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  let focus = $state(false);

  async function createEditor(paragraph: Paragraph) {
    const ydoc = await paragraph.ydoc();

    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions: [
        ...ParagraphSchema,
        Collabolation.configure({
          fragment: ydoc.getXmlFragment("doc"),
        }),
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
      },
      onCreate: () => {},
      onDestroy: () => {
        if (editor) {
          paragraph.doc = editor.getJSON();
          paragraph.save();
          editor = undefined;
        }
      },
      onFocus: () => {
        focus = true;
      },
    });
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
  class={editorBoxStyle}
  style:display={editor ? "block" : "none"}
  onmouseleave={() => {
    if (editor && !focus) {
      editor?.destroy();
    }
  }}
></div>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class={editorBoxStyle}
  style:display={editor ? "none" : "block"}
  onmouseenter={() => createEditor(paragraph)}
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
  const editorBoxStyle = css({
    w: "full",
    h: "fit",
    wordBreak: "break-word",
    minHeight: "[1.5rem]",
    "& p": {
      wordBreak: "break-word",
      color: "card.text",
      minHeight: "[1.5rem]",
    },
  });
</script>
