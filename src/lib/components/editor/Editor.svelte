<script lang="ts" generics="T extends Outline | Paragraph">
  import { Editor } from "@tiptap/core";
  import { onDestroy } from "svelte";
  import { css } from "styled-system/css";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import type { FocusPosition, EditorFocusPosition } from "./utils";
  import { watch } from "runed";
  import { createOutlineExtensions, createParagraphExtensions } from "./schema";
  import { Window } from "$lib/models/Window.svelte";
  import MockParagraphEditor from "./MockParagraphEditor.svelte";
  import { outlineEditorStyle, paragraphEditorStyle } from "./styles";
  import { Outline } from "$lib/models/Outline.svelte";
  import MockOutlineEditor from "./MockOutlineEditor.svelte";

  type EditorStyleVariants<T> = T extends Outline
    ? Parameters<typeof outlineEditorStyle>[0]
    : Parameters<typeof paragraphEditorStyle>[0];

  type Props = {
    doc: T;
    variant: EditorStyleVariants<T>;
    focusPosition: FocusPosition;
    isViewFocused: boolean;
  };

  let {
    doc,
    focusPosition = $bindable(),
    isViewFocused,
    variant,
  }: Props = $props();

  const isFocused = $derived(focusPosition.id === doc.id);

  let editor: Editor | undefined = $state();
  let editorElement: HTMLDivElement = $state() as HTMLDivElement;

  watch(
    [() => focusPosition, () => isViewFocused, () => Window.hasFocus()],
    () => {
      if (isViewFocused && focusPosition.id === doc.id) {
        if (editor) {
          editor.commands.focus(focusPosition.position);
        } else {
          (async () => await createEditor(doc, focusPosition.position))();
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

  async function createEditor(
    doc: Outline | Paragraph,
    pos: EditorFocusPosition,
  ) {
    if (editor) return;

    editor = new Editor({
      element: editorElement,
      extensions:
        doc instanceof Outline
          ? await createOutlineExtensions(doc, (pos) => (focusPosition = pos))
          : await createParagraphExtensions(
              doc,
              (pos) => (focusPosition = pos),
            ),
      editorProps: {
        attributes: {
          class: noRing,
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
          doc.save();
          editor = undefined;
        }
      },
      onFocus: () => {
        if (focusPosition.id !== doc.id) {
          focusPosition = {
            id: doc.id,
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
  style:display={editor ? "block" : "none"}
  class={doc instanceof Outline
    ? outlineEditorStyle(variant as EditorStyleVariants<Outline>)
    : paragraphEditorStyle(variant as EditorStyleVariants<Paragraph>)}
  onmouseleave={() => {
    if (editor && !isFocused) {
      editor?.destroy();
    }
  }}
></div>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="mock-editor"
  style:display={editor ? "none" : "block"}
  onmouseenter={() => createEditor(doc, null)}
>
  <div contenteditable tabindex="-1">
    {#if doc instanceof Outline}
      <MockOutlineEditor
        outline={doc}
        variant={variant as EditorStyleVariants<Outline>}
      />
    {:else}
      <MockParagraphEditor
        paragraph={doc}
        variant={variant as EditorStyleVariants<Paragraph>}
      />
    {/if}
  </div>
</div>

<script module>
  const noRing = css({
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
