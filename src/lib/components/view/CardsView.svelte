<script lang="ts">
  import { X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import DialogClose from "../common/DialogClose.svelte";
  import { buttonStyle } from "../common/Button.svelte";
  import OutlineEditor from "../editor/OutlineEditor.svelte";
  import { commands } from "../../../generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import Asterisk from "../icon/Asterisk.svelte";
  import ScrollArea from "../common/ScrollArea.svelte";
  import ParagraphEditor from "../editor/ParagraphEditor.svelte";
  import type { FocusPosition } from "../editor/utils";

  let { outlineId = $bindable() }: { outlineId?: string } = $props();

  let focusPosition: FocusPosition = $state.raw({ id: null, position: null });

  const promise = (async () => {
    return outlineId
      ? await commands
          .fetchTree(outlineId, 2)
          .then(unwrap)
          .then(([outlines, paragraphs]) =>
            Outline.tree(outlines, paragraphs, outlineId),
          )
      : await (async () => {
          const outline = await Outline.new();
          const paragraph = Paragraph.new(outline);
          outline.insertParagraph(paragraph);
          focusPosition = { id: paragraph.id, position: "start" };
          return outline;
        })();
  })();
</script>

<div class={viewContainer}>
  <div class={headerStyle}>
    <div class={headerLeftButtons}></div>
    <div class={headerRightButtons}>
      <DialogClose class={headerButtonStyle}>
        <X class={headerIconStyle} />
      </DialogClose>
    </div>
  </div>

  {#await promise then outline}
    <ScrollArea orientation="vertical" type="scroll">
      <div class={contentContainerStyle}>
        <div class={titleOutlineContainerStyle}>
          <div class={titleOutlineBulletContainerStyle}>
            <Asterisk
              class={titleOutlineAsteriskStyle}
              data-outline-empty={Boolean(outline.text.length)}
            />
            <div class={titleBulletBoxLine}></div>
          </div>
          <OutlineEditor
            {outline}
            bind:focusPosition
            containerStyle={titleOutlineEditorContainer}
            editorStyleVariant="cardsViewTitle"
          />
        </div>
        <div class={paragraphContainerStyle}>
          <div class={paragraphContainerLine}></div>
          {#each outline.paragraphs as paragraph (paragraph.id)}
            <div class={paragraphStyle}>
              <ParagraphEditor
                {paragraph}
                bind:focusPosition
                containerStyle={paragraphEditorContainer}
                editorStyleVariant="card"
              />
            </div>
          {/each}
        </div>
        <div class={contentBoxBottomSpace}>
          <div class={contetBoxBottomLine}></div>
          <div class={roundedLineEnd}></div>
        </div>
      </div>
    </ScrollArea>
  {/await}
</div>

<script module>
  const viewContainer = css({
    w: "full",
    h: "fit",
  });

  const headerStyle = css({
    position: "absolute",
    zIndex: "10",
    flexDir: "row",
    justifyContent: "space-between",
    alignItems: "center",
    w: "full",
    h: "8",
    px: "1",
    py: "1",
    bg: "transparent",
  });

  const headerLeftButtons = css({
    display: "flex",
    flexDir: "row",
    justifyContent: "flex-start",
  });

  const headerRightButtons = css({
    display: "flex",
    flexDir: "row",
    justifyContent: "flex-end",
  });

  const headerButtonStyle = css({
    ...buttonStyle,
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    shadow: "[none]",
    bg: "transparent",
    _hover: {
      bg: "selected",
    },
  });

  const headerIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });

  const contentContainerStyle = css({
    maxH: "[95vh]",
    px: "2",
    pt: "32",
  });

  const titleOutlineContainerStyle = css({
    display: "grid",
    gridTemplateColumns: "2.5rem 1fr",
    position: "relative",
    w: "full",
  });

  const titleOutlineBulletContainerStyle = css({
    position: "absolute",
    left: "0",
    top: "0",
    w: "10",
    h: "full",
  });

  const titleOutlineAsteriskStyle = css({
    w: "6",
    h: "6",
    fill: "view.text",
    position: "relative",
    top: "3",
    left: "0",
    transition: "colors",
    "&[data-outline-empty=false]": {
      fill: "view.text-muted",
    },
  });

  const titleBulletBoxLine = css({
    w: "[0.0625rem]",
    h: "[calc(100% - 2.5rem)]",
    position: "absolute",
    top: "10",
    left: "[0.75rem]",
    bg: "view.text-muted",
    rounded: "md",
  });

  const contentBoxBottomSpace = css({
    h: "32",
    w: "full",
    position: "relative",
  });

  const contetBoxBottomLine = css({
    w: "[0.0625rem]",
    h: "[1.125rem]",
    position: "absolute",
    top: "0",
    left: "[0.75rem]",
    bg: "view.text-muted",
    rounded: "md",
  });

  const roundedLineEnd = css({
    w: "1",
    h: "1",
    bg: "view.text-muted",
    rounded: "circle",
    position: "absolute",
    top: "[1.125rem]",
    left: "[0.65rem]",
  });

  const paragraphContainerStyle = css({
    position: "relative",
    display: "flex",
    flexDir: "column",
    h: "fit",
    w: "full",
    pt: "4",
    gap: "4",
  });

  const paragraphContainerLine = css({
    w: "[0.0625rem]",
    h: "full",
    position: "absolute",
    z: "0",
    top: "0",
    left: "[0.75rem]",
    bg: "view.text-muted",
  });

  const paragraphStyle = css({
    position: "relative",
    z: "10",
    display: "flex",
    py: "[1.3125rem]",
    px: "[1.75rem]",
    bg: "card.bg",
    rounded: "lg",
    w: "full",
    h: "fit",
    shadow: "sm",
  });

  const titleOutlineEditorContainer = css.raw({
    w: "full",
    h: "fit",
    wordBreak: "break-word",
    gridColumn: "2",
    minHeight: "[3rem]",
    color: "view.text",
  });

  const paragraphEditorContainer = css.raw({
    w: "full",
    h: "fit",
    minHeight: "[1.5rem]",
  });
</script>
