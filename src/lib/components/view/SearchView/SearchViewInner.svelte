<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import { Editor } from "@tiptap/core";
  import { onMount, onDestroy } from "svelte";
  import { createSearchQueryExtensions } from "$lib/components/editor/schema";
  import Header from "../common/Header.svelte";
  import { ChevronDown, ChevronRight, SearchIcon, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";
  import ScrollArea from "$lib/components/common/ScrollArea.svelte";
  import Asterisk from "$lib/components/icon/Asterisk.svelte";
  import VerticalLine from "$lib/components/icon/VerticalLine.svelte";
  import VerticalLineDash from "$lib/components/icon/VerticalLineDash.svelte";
  import Tilda from "$lib/components/icon/Tilda.svelte";
  import HoverViewTriggerParagraph from "../common/HoverViewTriggerParagraph.svelte";
  import VerticalLineWithCircle from "$lib/components/icon/VerticalLineWithCircle.svelte";
  import HoverViewContext from "../common/HoverViewContext.svelte";
  import Popover from "$lib/components/common/Popover.svelte";
  import PopoverClose from "$lib/components/common/PopoverClose.svelte";
  import type { View } from "$lib/models/Workspace.svelte";

  type Props = { view: View<"search">; search: Search; pinned: boolean };

  let { view, search, pinned }: Props = $props();

  let scrollAreaRef: HTMLDivElement = $state()!;
  let queryElement: HTMLDivElement = $state()!;
  let queryEditor: Editor;

  let isComposing = false;

  const REM = 16;

  onMount(() => {
    queryEditor = new Editor({
      element: queryElement,
      extensions: createSearchQueryExtensions(),
      content: search.query,
      onUpdate: () => {
        if (scrollAreaRef.scrollTop > 8 * REM) {
          scrollAreaRef.scrollTo({ top: 8 * REM });
        }

        if (!isComposing) {
          search.query = queryEditor.getText();
        }
      },
      editorProps: {
        attributes: {
          class: noRing,
        },
      },
    });

    queryElement.addEventListener("compositionstart", () => {
      isComposing = true;
    });

    queryElement.addEventListener("compositionend", () => {
      isComposing = false;
      search.query = queryEditor.getText();
    });
  });

  onDestroy(() => {
    queryEditor.destroy();
    search.cleanup();
  });
</script>

<Header>
  {#snippet right({ buttonStyle, iconStyle })}
    {#if !pinned}
      <Button class={css(buttonStyle)}>
        <X class={css(iconStyle)} />
      </Button>
    {/if}
  {/snippet}
  {#snippet center({ buttonStyle, iconStyle, textStyle })}
    <Button class={css(buttonStyle)}>
      <SearchIcon class={css(iconStyle)} />
    </Button>
    <div class={css(textStyle)}>Search</div>
  {/snippet}
</Header>

<ScrollArea bind:ref={scrollAreaRef} type="always">
  <div class={contentContainerStyle}>
    <div class={queryAreaStyle}>
      {#if search.path}
        {#await search.path then path}
          <div class={searchScopeStyle}>
            <div class={searchScopeTextStyle}>In:</div>
            <div class={searchScopePathStyle}>
              {#each path as pathItem, idx}
                {#if idx !== 0}
                  <ChevronRight class={searchScopePathIconStyle} />
                {/if}
                <div class={searchScopePathTextStyle}>
                  {pathItem.text}
                </div>
              {/each}
            </div>
          </div>
        {/await}
      {/if}
      <div class={queryEditorContainerStyle}>
        <div class={queryEditorIconContainerStyle}>
          <SearchIcon class={queryEditorIconStyle} />
        </div>
        <div bind:this={queryElement} class={queryEditorStyle}></div>
      </div>
      <div class={searchOptionStyle}>
        <Popover
          triggerProps={{ class: searchOptionButtonStyle }}
          contentProps={{
            align: "end",
          }}
          contentStyle={css.raw({
            bg: "card.bg",
            display: "flex",
            flexDir: "column",
          })}
        >
          {#snippet trigger()}
            <ChevronDown class={searchOptionIconStyle} />
            {#if view.orderBy === "relevance"}
              <div class={searchOptionTextStyle}>Relevance</div>
            {:else if "createdAt" in view.orderBy}
              <div class={searchOptionTextStyle}>
                {#if view.orderBy.createdAt === "asc"}
                  Created At: asc
                {:else}
                  Created At: desc
                {/if}
              </div>
            {:else}
              <div class={searchOptionTextStyle}>
                {#if view.orderBy.updatedAt === "asc"}
                  Updated At: asc
                {:else}
                  Updated At: desc
                {/if}
              </div>
            {/if}
          {/snippet}
          {#snippet content()}
            <PopoverClose
              onclick={() => (view.orderBy = { createdAt: "asc" })}
              class={searchOptionButtonStyle}
            >
              <div class={searchOptionTextStyle}>
                Created At: asc
              </div></PopoverClose
            >
            <PopoverClose
              onclick={() => (view.orderBy = { updatedAt: "asc" })}
              class={searchOptionButtonStyle}
            >
              <div class={searchOptionTextStyle}>
                Updated At: asc
              </div></PopoverClose
            >
            <PopoverClose
              onclick={() => (view.orderBy = { createdAt: "desc" })}
              class={searchOptionButtonStyle}
            >
              <div class={searchOptionTextStyle}>
                Created At: desc
              </div></PopoverClose
            >
            <PopoverClose
              onclick={() => (view.orderBy = { updatedAt: "desc" })}
              class={searchOptionButtonStyle}
            >
              <div class={searchOptionTextStyle}>
                Updated At: desc
              </div></PopoverClose
            >
            <PopoverClose
              onclick={() => (view.orderBy = "relevance")}
              class={searchOptionButtonStyle}
            >
              <div class={searchOptionTextStyle}>Relevance</div></PopoverClose
            >
          {/snippet}
        </Popover>
      </div>
    </div>

    <div class={searchResultsContainer}>
      <HoverViewContext>
        {#await search.result then result}
          {#each result as { outline, paragraphs }}
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
                      <div
                        class={pathTextStyle}
                        data-last={path.length - 1 === idx}
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
                      {#if paragraph.id in search.paragraphPositionIndex}
                        {@const index =
                          search.paragraphPositionIndex[paragraph.id]}
                        {#if index && (index.prevId === paragraphs[idx - 1]?.id || index.prevId === null)}
                          <VerticalLine class={paragraphContainerLineTop} />
                        {:else}
                          <VerticalLineDash class={paragraphContainerLineTop} />
                          <div class={tildaTopContainer}>
                            <Tilda class={tildaTop} />
                          </div>
                        {/if}
                      {/if}
                      <HoverViewTriggerParagraph {paragraph} />
                      {#if idx === paragraphs.length - 1}
                        {@const index =
                          search.paragraphPositionIndex[paragraph.id]}
                        {#if index?.isLast}
                          <VerticalLineWithCircle
                            class={paragraphContainerLineBottom}
                          />
                        {:else}
                          <VerticalLineDash
                            class={paragraphContainerLineBottom}
                          />
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
        {/await}
      </HoverViewContext>
    </div>
  </div>
</ScrollArea>

<script module>
  const contentContainerStyle = css({
    display: "flex",
    flexDir: "column",
    maxW: "[38.25rem]",
    py: "32",
    m: "auto",
    "&[data-loading=true]": {
      opacity: "0",
    },
  });

  const queryAreaStyle = css({
    position: "sticky",
    top: "0",
    zIndex: "[10]",
    display: "flex",
    flexDir: "column",
    gap: "1",
    w: "full",
    h: "fit",
    px: "1",
    bgGradient: "to-b",
    gradientFrom: "view.bg",
    gradientVia: "view.bg",
    gradientTo: "view.bg/0",
  });

  const searchScopeStyle = css({
    display: "grid",
    gridTemplateColumns: "[3rem 1fr]",
    w: "full",
    h: "fit",
  });

  const searchScopeTextStyle = css({
    mx: "auto",
    color: "view.text-muted",
    fontSize: "sm",
    fontFamily: "mono",
    fontWeight: "light",
  });

  const searchScopePathStyle = css({
    display: "flex",
    flexDir: "row",
    alignItems: "center",
    flexWrap: "wrap",
    w: "full",
    h: "fit",
    gap: "1",
  });

  const searchScopePathTextStyle = css({
    color: "view.text-muted",
    fontSize: "sm",
    textWrap: "wrap",
    wordBreak: "break-all",
  });

  const searchScopePathIconStyle = css({
    w: "3",
    h: "3",
    color: "view.text-muted",
  });

  const searchOptionStyle = css({
    display: "flex",
    justifyContent: "end",
    w: "full",
    h: "fit",
    pr: "2",
  });

  const searchOptionButtonStyle = css({
    display: "flex",
    gap: "2",
    rounded: "md",
    px: "1",
    transition: "colors",
    _hover: {
      bg: "selected",
    },
  });

  const searchOptionIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });

  const searchOptionTextStyle = css({
    fontSize: "xs",
    fontFamily: "mono",
    color: "view.text-muted",
  });

  const queryEditorContainerStyle = css({
    display: "grid",
    gridTemplateColumns: "[1.5rem 1fr]",
    gap: "2",
    rounded: "lg",
    w: "full",
    h: "fit",
    py: "2",
    px: "4",
    bg: "card.bg",
    shadow: "md",
  });

  const queryEditorIconContainerStyle = css({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    w: "6",
    h: "6",
  });

  const queryEditorIconStyle = css({
    w: "4",
    h: "4",
    color: "card.text-muted",
  });

  const queryEditorStyle = css({
    w: "full",
    h: "fit",
    minH: "[1.6rem]",
    "& p": {
      minH: "[1.6rem]",
      color: "card.text",
      userSelect: "text",
    },
  });

  const noRing = css({
    ring: "none",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
  });

  const searchResultsContainer = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    px: "2",
    gap: "4",
  });

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
    color: "view.text-muted",
    fontSize: "md",
    textWrap: "wrap",
    wordBreak: "break-all",
    "&[data-last=true]": {
      color: "view.text",
      fontWeight: "bold",
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
</script>
