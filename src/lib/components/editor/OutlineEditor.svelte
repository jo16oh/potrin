<script lang="ts">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css, type Styles } from "styled-system/css";
  import type { Outline } from "$lib/models/Outline.svelte";
  import type { FocusPosition, EditorFocusPosition } from "./utils";
  import { watch } from "runed";
  import { createOutlineExtensions } from "./schema";
  import { Window } from "$lib/models/Window.svelte";

  type EditorStyleVariant = "cardsViewTitle" | "cardsViewChildren";

  type Props = {
    outline: Outline;
    focusPosition: FocusPosition;
    isViewFocused: boolean;
    containerStyle?: Styles;
    editorStyleVariant: EditorStyleVariant;
  };

  let {
    outline,
    focusPosition = $bindable(),
    isViewFocused,
    containerStyle,
    editorStyleVariant,
  }: Props = $props();

  const isFocused = $derived(focusPosition.id === outline.id);

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  watch(
    [() => focusPosition, () => isViewFocused, () => Window.hasFocus()],
    () => {
      if (isViewFocused && focusPosition.id === outline.id) {
        if (editor) {
          editor.commands.focus(focusPosition.position);
        } else {
          (async () => await createEditor(outline, focusPosition.position))();
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

  async function createEditor(outline: Outline, pos: EditorFocusPosition) {
    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions: [
        ...(await createOutlineExtensions(
          outline,
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
        if (isFocused) {
          // property assignment doesn't trigger effect when using `watch`
          focusPosition.position = editor!.state.selection.from;
        }
      },
      onDestroy: () => {
        if (editor) {
          outline.save();
          editor = undefined;
        }
      },
      onFocus: () => {
        if (!isFocused) {
          focusPosition = {
            id: outline.id,
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
  onmouseenter={() => createEditor(outline, null)}
>
  <div class="mock-editor">
    <div contenteditable tabindex="-1" class="tiptap ProseMirror">
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
  </div>
</div>

<script module>
  const editorStyleVariants = {
    cardsViewTitle: css.raw({
      "& p": {
        fontSize: "[2rem]",
        fontWeight: "semibold",
        color: "view.text",
      },
    }),
    cardsViewChildren: css.raw({
      "& p": {
        fontSize: "[2rem]",
        fontWeight: "semibold",
        color: "view.text",
      },
    }),
  };

  const noRing = css.raw({
    ring: "none",
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
