<script lang="ts">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css, type Styles } from "styled-system/css";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import type { FocusPosition, EditorFocusPosition } from "./utils";
  import { watch } from "runed";
  import { createParagraphExtensions } from "./schema";
  import { Window } from "$lib/models/Window.svelte";

  type EditorStyleVariant = "card";

  type Props = {
    paragraph: Paragraph;
    focusPosition: FocusPosition;
    isViewFocused: boolean;
    containerStyle?: Styles;
    editorStyleVariant: EditorStyleVariant;
  };

  let {
    paragraph,
    focusPosition = $bindable(),
    isViewFocused,
    containerStyle,
    editorStyleVariant,
  }: Props = $props();

  const isFocused = $derived(focusPosition.id === paragraph.id);

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  watch(
    [() => focusPosition, () => isViewFocused, () => Window.hasFocus()],
    () => {
      if (isViewFocused && focusPosition.id === paragraph.id) {
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

  watch(
    () => editor,
    () => {
      setTimeout(() => {
        if (!editor && isFocused && isViewFocused && Window.hasFocus()) {
          focusPosition.id = null;
          focusPosition.position = null;
        }
      });
    },
    { lazy: true },
  );

  async function createEditor(paragraph: Paragraph, pos: EditorFocusPosition) {
    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions: [
        ...(await createParagraphExtensions(
          paragraph,
          (pos) => (focusPosition = pos),
        )),
      ],
      editorProps: {
        attributes: {
          class: css(noRing),
        },
      },
      onBlur: () => {
        setTimeout(() => {
          editor?.destroy();
        });
      },
      onTransaction: () => {
        if (focusPosition.id === paragraph.id) {
          // property assignment doesn't trigger effect when using `watch`
          focusPosition.position = editor!.state.selection.from;
        }
      },
      onDestroy: () => {
        if (editor) {
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
  class={css(containerStyle, editorStyleVariants[editorStyleVariant], noRing)}
  style:display={editor ? "none" : "block"}
  onmouseenter={() => createEditor(paragraph, null)}
>
  <div class="mock-editor">
    <div contenteditable tabindex="-1" class="tiptap ProseMirror">
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
  </div>
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

  const noRing = css.raw({
    ring: "none",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
  });
</script>

<style>
  .mock-editor {
    position: relative;
  }

  .mock-editor::after {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }
</style>
