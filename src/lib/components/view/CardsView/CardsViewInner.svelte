<script lang="ts">
  import { ChevronLeft, ChevronRight, Search, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";
  import { Outline } from "$lib/models/Outline.svelte";
  import Asterisk from "$lib/components/icon/Asterisk.svelte";
  import ScrollArea from "$lib/components/common/ScrollArea.svelte";
  import Editor from "$lib/components/editor/Editor.svelte";
  import { debounce } from "es-toolkit";
  import { watch } from "runed";
  import { onMount } from "svelte";
  import CardStack from "$lib/components/icon/CardStack.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import VerticalLine from "$lib/components/icon/VerticalLine.svelte";
  import VerticalLineWithCircle from "$lib/components/icon/VerticalLineWithCircle.svelte";
  import Header from "../common/Header.svelte";

  type Props = {
    outline: Outline;
    view: View<"cards">;
    isFocused: boolean;
    pinned: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    outline,
    view = $bindable(),
    isFocused,
    pinned,
    onCloseButtonClick,
  }: Props = $props();

  const REM = 16;

  let scrollAreaRef = $state<HTMLDivElement>()!;

  watch(
    () => outline.text,
    debounce(() => (view.title = outline.text), 400),
  );

  onMount(() => {
    // If the view is opend in hoverview on timeline,
    // scroll to the point where the cursor is visible
    if (view.focusPosition?.id && view.focusPosition.position === "end") {
      const editor = document.getElementById(view.id + view.focusPosition.id);
      if (!editor) return;
      const editorRect = editor.getBoundingClientRect();
      const scrollRect = scrollAreaRef.getBoundingClientRect();

      if (
        editorRect.bottom > scrollRect.bottom ||
        editorRect.bottom < editorRect.top + scrollAreaRef.clientHeight / 2
      ) {
        // If the editor's height is bigger than viewport's height,
        // scroll to the bottom of the editor.
        if (scrollAreaRef.clientHeight < editor.clientHeight) {
          editor.scrollIntoView({ block: "end" });
        }
        // else, scroll to the top of the editor.
        else {
          editor.scrollIntoView({ block: "start" });
          scrollAreaRef.scrollTo({
            top: scrollAreaRef.scrollTop - 2 * (REM / 4),
          });
        }
      }
    }
    // else, restore previous scroll position
    else {
      scrollAreaRef?.scrollTo(0, view.scrollPosition);
    }
  });

  const onscroll = debounce(() => {
    view.scrollPosition = scrollAreaRef?.scrollTop ?? 0;
  }, 100);
</script>

<Header>
  {#snippet left({ buttonStyle, iconStyle })}
    <Button
      class={css(buttonStyle)}
      disabled={!View.hasPrev(view.id)}
      onmousedown={(e: MouseEvent) => e.preventDefault()}
      onclick={(e: MouseEvent) => {
        e.preventDefault();
        View.back(view);
      }}
    >
      <ChevronLeft
        class={css(iconStyle)}
        data-disabled={!View.hasPrev(view.id)}
      />
    </Button>
    <Button
      class={css(buttonStyle)}
      disabled={!View.hasNext(view.id)}
      onmousedown={(e: MouseEvent) => e.preventDefault()}
      onclick={(e: MouseEvent) => {
        e.preventDefault();
        View.forward(view);
      }}
    >
      <ChevronRight
        class={css(iconStyle)}
        data-disabled={!View.hasNext(view.id)}
      />
    </Button>
    <Button class={css(buttonStyle)}>
      <Search
        class={css(iconStyle)}
        onmousedown={(e: MouseEvent) => e.preventDefault()}
        onclick={(e: MouseEvent) => {
          e.preventDefault();
          View.open(view, { ...View.new("search"), scope: outline.id });
        }}
      />
    </Button>
  {/snippet}
  {#snippet center({ buttonStyle, iconStyle, textStyle })}
    <Button class={css(buttonStyle)}>
      <CardStack class={css(iconStyle)} />
    </Button>
    <div class={css(textStyle)}>
      {#if view.title.length === 0}
        Untitled
      {:else}
        {view.title}
      {/if}
    </div>
  {/snippet}
  {#snippet right({ buttonStyle, iconStyle })}
    {#if !pinned}
      <Button class={css(buttonStyle)} onclick={onCloseButtonClick}>
        <X class={css(iconStyle)} />
      </Button>
    {/if}
  {/snippet}
</Header>

<ScrollArea
  bind:ref={scrollAreaRef}
  type="always"
  orientation="vertical"
  {onscroll}
>
  <div class={contentContainerStyle}>
    <div class={titleOutlineContainerStyle}>
      <div class={titleOutlineBulletContainerStyle}>
        <Asterisk
          class={titleOutlineAsteriskStyle}
          data-outline-empty={Boolean(outline.text.length)}
        />
        <VerticalLine class={titleBulletBoxLine} />
      </div>
      <Editor
        doc={outline}
        isViewFocused={isFocused}
        bind:focusPosition={view.focusPosition}
        variant={{ style: "cardsViewTitle" }}
      />
    </div>
    <div class={paragraphContainerStyle}>
      <VerticalLine class={paragraphContainerLine} />
      {#each outline.paragraphs as paragraph (paragraph.id)}
        <div id={view.id + paragraph.id}>
          <Editor
            doc={paragraph}
            variant={{ style: "card" }}
            isViewFocused={isFocused}
            bind:focusPosition={view.focusPosition}
          />
        </div>
      {/each}
    </div>
    <div class={contentBoxBottomSpace}>
      <!-- <div class={contetBoxBottomLine}></div> -->
      <VerticalLineWithCircle class={contetBoxBottomLine} />
    </div>
  </div>
</ScrollArea>

<script module>
  const contentContainerStyle = css({
    maxW: "[38.25rem]",
    px: "2",
    pt: "32",
    m: "auto",
  });

  const titleOutlineContainerStyle = css({
    display: "grid",
    gridTemplateColumns: "2.5rem 1fr",
    position: "relative",
    w: "full",
  });

  const titleOutlineBulletContainerStyle = css({
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
    w: "6",
    h: "[calc(100% - 2.5rem)]",
    position: "absolute",
    top: "10",
    color: "view.text-muted",
  });

  const contentBoxBottomSpace = css({
    h: "32",
    w: "full",
    position: "relative",
  });

  const contetBoxBottomLine = css({
    w: "6",
    h: "[1.125rem]",
    position: "absolute",
    top: "0",
    color: "view.text-muted",
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
    w: "6",
    h: "full",
    position: "absolute",
    z: "-1",
    top: "0",
    color: "view.text-muted",
  });
</script>
