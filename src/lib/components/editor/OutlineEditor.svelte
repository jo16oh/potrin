<script lang="ts">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css } from "styled-system/css";
  import type { Outline } from "$lib/models/Outline.svelte";
  import Collabolation from "@tiptap/extension-collaboration";
  import { OutlineSchema } from "./schema";

  type Props = { outline: Outline };

  let { outline }: Props = $props();

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  let focus = $state(false);

  async function createEditor(outline: Outline) {
    const ydoc = await outline.ydoc();

    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions: [
        ...OutlineSchema,
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
        outline.text = editor!.getText();
      },
      onBlur: () => {
        focus = false;
        setTimeout(() => {
          editor?.destroy();
        });
      },
      onCreate: () => {},
      onDestroy: () => {
        if (editor) {
          outline.doc = editor.getJSON();
          outline.text = editor.getText();
          outline.save();
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
  onmouseenter={() => createEditor(outline)}
>
  {#if outline.doc}
    {#each outline.doc.content ?? [] as content}
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
    gridColumn: "2",
    minHeight: "[3rem]",
    color: "view.text",
    "& p": {
      wordBreak: "break-word",
      fontSize: "[2rem]",
      fontWeight: "semibold",
      color: "view.text",
    },
  });
</script>
