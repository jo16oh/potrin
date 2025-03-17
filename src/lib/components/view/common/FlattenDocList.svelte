<script lang="ts">
  import type { Outline } from "$lib/models/Outline.svelte";
  import type { Paragraph } from "$lib/models/Paragraph.svelte";
  import { ChevronRight } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Asterisk from "$lib/components/icon/Asterisk.svelte";
  import VerticalLine from "$lib/components/icon/VerticalLine.svelte";
  import VerticalLineDash from "$lib/components/icon/VerticalLineDash.svelte";
  import Tilda from "$lib/components/icon/Tilda.svelte";
  import VerticalLineWithCircle from "$lib/components/icon/VerticalLineWithCircle.svelte";
  import type { ParagraphPositionIndex } from "generated/tauri-commands";
  import type { SvelteHTMLElements } from "svelte/elements";
  import Button from "$lib/components/common/Button.svelte";
  import HoverView from "$lib/components/common/HoverView";
  import MockParagraphEditor from "$lib/components/editor/MockParagraphEditor.svelte";
  import { View } from "$lib/models/Workspace.svelte";

  type FlattenDocListItem = {
    outline: Outline;
    paragraphs: Paragraph[];
  };

  type Props = {
    view: View<"timeline" | "search">;
    items: FlattenDocListItem[];
    paragraphPositionIndex: ParagraphPositionIndex;
    openLinkOnHover: boolean;
  } & SvelteHTMLElements["div"];

  let {
    view,
    items,
    paragraphPositionIndex,
    openLinkOnHover,
    ...restProps
  }: Props = $props();

  const hoverViewContext = HoverView.State.current;

  function open(outlineId: string, focusedId: string) {
    if (hoverViewContext && openLinkOnHover) {
      hoverViewContext.view = {
        ...View.new("cards"),
        outlineId: outlineId,
        focusPosition: { id: focusedId, position: "end" },
      };
      hoverViewContext.open = true;
    } else {
      View.open(view, {
        ...View.new("cards"),
        outlineId,
        focusPosition: { id: focusedId, position: "end" },
      });
    }
  }
</script>

<div {...restProps}>
  {#each items as { outline, paragraphs }}
    <div class={outlineContainerStyle}>
      <div class={outlineStyle}>
        <div class={asteriskContainerStyle}>
          <Asterisk class={asteriskStyle} />
        </div>
        <div class={pathStyle}>
          {#await outline.path then path}
            {#each path as pathItem, idx}
              {#if idx !== 0}
                <ChevronRight class={chevronStyle} />
              {/if}
              <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
              <div
                class={pathTextStyle}
                data-last={path.length - 1 === idx}
                data-open-link-on-hover={openLinkOnHover ? true : false}
                onclick={() => open(pathItem.id, pathItem.id)}
              >
                {pathItem.text}
              </div>
            {/each}
          {/await}
        </div>
      </div>

      {#if paragraphs.length}
        <div class={paragraphContainerStyle}>
          {#each paragraphs as paragraph, idx (paragraph.id)}
            <div class={paragraphStyle}>
              {#if paragraph.id in paragraphPositionIndex}
                {@const index = paragraphPositionIndex[paragraph.id]}
                {#if index && (index.prevId === paragraphs[idx - 1]?.id || index.prevId === null)}
                  <VerticalLine class={paragraphContainerLineTop} />
                {:else}
                  <VerticalLineDash class={paragraphContainerLineTop} />
                  <div class={tildaTopContainer}>
                    <Tilda class={tildaTop} />
                  </div>
                {/if}
              {:else}
                <VerticalLine class={paragraphContainerLineTop} />
              {/if}
              <Button
                class={paragraphTriggerStyle}
                data-open-link-on-hover={openLinkOnHover ? true : false}
                onclick={() => open(paragraph.outlineId, paragraph.id)}
              >
                <MockParagraphEditor {paragraph} variant={{ style: "card" }} />
              </Button>
              {#if idx === paragraphs.length - 1}
                {@const index = paragraphPositionIndex[paragraph.id]}
                {#if index?.isLast}
                  <VerticalLineWithCircle
                    class={paragraphContainerLineBottom}
                  />
                {:else}
                  <VerticalLineDash class={paragraphContainerLineBottom} />
                  <div class={tildaBottomContainer}>
                    <Tilda class={tildaBottom} />
                  </div>
                {/if}
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

<script module>
  const outlineContainerStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
  });

  const outlineStyle = css({
    pl: "[0.00625rem]",
    display: "flex",
    flexDir: "row",
    alignItems: "start",
    gap: "1",
    w: "full",
    h: "fit",
  });

  const asteriskContainerStyle = css({
    w: "6",
    h: "6",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  });

  const asteriskStyle = css({
    w: "3.5",
    h: "3.5",
    fill: "view.text-muted",
  });

  const pathStyle = css({
    display: "flex",
    flexDir: "row",
    alignItems: "center",
    flexWrap: "wrap",
    w: "full",
    h: "fit",
    gap: "1",
  });

  const pathTextStyle = css({
    textAlign: "start",
    color: "view.text-muted",
    fontSize: "md",
    textWrap: "wrap",
    wordBreak: "break-all",
    "&[data-last=true]": {
      color: "view.text",
      fontWeight: "bold",
    },
    _hover: {
      textDecoration: "underline",
      textDecorationColor: "view.text",
    },
    "&[data-open-link-on-hover=true]": {
      _hover: {
        cursor: "alias",
      },
    },
    "&[data-open-link-on-hover=false]": {
      _hover: {
        cursor: "pointer",
      },
    },
  });

  const paragraphContainerStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    py: "4",
    gap: "4",
  });

  const chevronStyle = css({
    color: "view.text-muted",
    w: "4",
    h: "4",
  });

  const paragraphContainerLineTop = css({
    h: "4",
    w: "6",
    position: "absolute",
    z: "-10",
    top: "-4",
    color: "view.text-muted",
  });

  const paragraphContainerLineBottom = css({
    h: "4",
    w: "6",
    position: "absolute",
    z: "-10",
    bottom: "-4",
    color: "view.text-muted",
  });

  const paragraphStyle = css({
    w: "full",
    h: "fit",
    position: "relative",
  });

  const tildaTopContainer = css({
    h: "4",
    w: "6",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    position: "absolute",
    z: "-10",
    top: "-4",
    color: "view.text-muted",
  });

  const tildaBottomContainer = css({
    h: "4",
    w: "6",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    position: "absolute",
    z: "-10",
    bottom: "-4",
    color: "view.text-muted",
  });

  const tildaTop = css({
    h: "2.5",
    w: "2.5",
    color: "view.text-muted",
  });

  const tildaBottom = css({
    h: "2.5",
    w: "2.5",
    color: "view.text-muted",
  });

  const paragraphTriggerStyle = css({
    w: "full",
    h: "fit",
    textAlign: "start",
    rounded: "lg",
    _hover: {
      outlineColor: "[blue]",
      outlineOffset: "1",
      outlineStyle: "solid",
      outlineWidth: "thin",
    },
    "&[data-open-link-on-hover=true]": {
      _hover: {
        cursor: "alias",
      },
    },
    "&[data-open-link-on-hover=false]": {
      _hover: {
        cursor: "pointer",
      },
    },
  });
</script>
