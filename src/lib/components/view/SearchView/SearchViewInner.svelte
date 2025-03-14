<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import { Editor } from "@tiptap/core";
  import { onMount, onDestroy, tick } from "svelte";
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
  import { watch } from "runed";
  import { debounce } from "es-toolkit";

  type Props = {
    view: View<"search">;
    search: Search;
    pinned: boolean;
    onCloseButtonClick: () => void;
  };

  let { view, search, pinned, onCloseButtonClick }: Props = $props();

  let scrollAreaRef: HTMLDivElement = $state()!;
  let queryElement: HTMLDivElement = $state()!;
  let queryEditor: Editor;

  let isComposing = false;

  const REM = 16;

  watch(
    () => [search.path, search.query],
    () => {
      const syncTitle = async () => {
        const path = await search.path;
        view.title = path
          ? "In:" + path.map((p) => p.text).join("/") + " " + search.query
          : search.query;
      };

      if (!view.title.length) {
        syncTitle();
      } else {
        debounce(syncTitle, 400)();
      }
    },
  );

  onMount(() => {
    queryEditor = new Editor({
      element: queryElement,
      autofocus: "all",
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

  let contentElement: HTMLDivElement = $state()!;

  type HighlightOptions = {
    query?: string;
    disabled?: boolean;
    node_filter?: (node: Node) => number;
    cssClass?: string;
  };

  watch(
    () => [search.query, search.result],
    () => {
      highlight(contentElement, { query: search.query });
    },
  );

  async function highlight(node: Node, ops: HighlightOptions) {
    await tick();

    const {
      query = ``,
      disabled = false,
      node_filter = () => NodeFilter.FILTER_ACCEPT,
      cssClass = `highlight-match`,
    } = ops;

    // clear previous ranges from HighlightRegistry
    CSS.highlights.clear();

    if (!query || disabled || typeof CSS == `undefined` || !CSS.highlights)
      return; // abort if CSS highlight API not supported

    // TODO: support more complex query
    const queries = query
      .toLowerCase()
      .split(/[\s\p{Zs}]+/u)
      .filter((q) => q.length > 0);
    if (queries.length === 0) return;

    const treeWalker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, {
      acceptNode: node_filter,
    });
    const textNodes: Node[] = [];
    let currentNode = treeWalker.nextNode();
    while (currentNode) {
      textNodes.push(currentNode);
      currentNode = treeWalker.nextNode();
    }

    const ranges = textNodes.flatMap((el) => {
      const text = el.textContent?.toLowerCase();
      if (!text) return [];

      return queries.flatMap((currentQuery) => {
        const indices = [];
        let startPos = 0;

        while (startPos < text.length) {
          const index = text.indexOf(currentQuery, startPos);
          if (index === -1) break;
          indices.push(index);
          startPos = index + currentQuery.length;
        }

        return indices.map((index) => {
          const range = new Range();
          range.setStart(el, index);
          range.setEnd(el, index + currentQuery.length);
          return range;
        });
      });
    });

    CSS.highlights.set(cssClass, new Highlight(...ranges));
  }
</script>

<Header>
  {#snippet center({ buttonStyle, iconStyle, textStyle })}
    <Button class={css(buttonStyle)}>
      <SearchIcon class={css(iconStyle)} />
    </Button>
    <div class={css(textStyle)}>Search</div>
  {/snippet}
  {#snippet right({ buttonStyle, iconStyle })}
    {#if !pinned}
      <Button class={css(buttonStyle)} onclick={onCloseButtonClick}>
        <X class={css(iconStyle)} />
      </Button>
    {/if}
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

    <div bind:this={contentElement}>
      <FlattenDocList
        class={searchResultsContainer}
        items={search.result}
        paragraphPositionIndex={search.paragraphPositionIndex}
      />
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
    userSelect: "none",
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

<style>
  :global {
    ::highlight(highlight-match) {
      background-color: yellow;
    }
  }
</style>
