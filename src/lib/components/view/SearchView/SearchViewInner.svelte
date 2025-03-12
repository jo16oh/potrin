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
  import Popover from "$lib/components/common/Popover.svelte";
  import PopoverClose from "$lib/components/common/PopoverClose.svelte";
  import type { View } from "$lib/models/Workspace.svelte";
  import FlattenDocList from "../common/FlattenDocList.svelte";

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

  function onscroll() {
    if (
      scrollAreaRef.scrollTop ===
      scrollAreaRef.scrollHeight - scrollAreaRef.clientHeight
    ) {
      search.loadMore();
    }
  }
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

<ScrollArea bind:ref={scrollAreaRef} type="always" {onscroll}>
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

    <FlattenDocList
      class={searchResultsContainer}
      items={search.result}
      paragraphPositionIndex={search.paragraphPositionIndex}
    />
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
</script>
