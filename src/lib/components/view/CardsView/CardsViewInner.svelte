<script lang="ts">
  import { ChevronLeft, ChevronRight, Search, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";
  import { type ViewState } from "generated/tauri-commands";
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

  type CardsViewState = Extract<ViewState, { type: "cards" }>;

  type Props = {
    outline: Outline;
    viewState: CardsViewState;
    isFocused: boolean;
    pinned: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    outline,
    viewState = $bindable(),
    isFocused,
    pinned,
    onCloseButtonClick,
  }: Props = $props();

  let scrollAreaRef = $state<HTMLDivElement>();

  watch(
    () => outline.text,
    debounce(() => (viewState.title = outline.text), 16),
  );

  onMount(() => {
    scrollAreaRef?.scrollTo(0, viewState.scrollPosition);
  });

  const onscroll = debounce(() => {
    if (scrollAreaRef) viewState.scrollPosition = scrollAreaRef?.scrollTop ?? 0;
  }, 100);
</script>

<ScrollArea bind:ref={scrollAreaRef} orientation="vertical" {onscroll}>
  <div class={headerStyle}>
    <div class={headerLeftButtons}>
      <Button
        class={headerButtonStyle}
        disabled={!View.hasPrev(viewState.id)}
        onmousedown={(e: MouseEvent) => e.preventDefault()}
        onclick={(e: MouseEvent) => {
          e.preventDefault();
          View.back(viewState);
        }}
      >
        <ChevronLeft
          class={headerIconStyle}
          data-disabled={!View.hasPrev(viewState.id)}
        />
      </Button>
      <Button
        class={headerButtonStyle}
        disabled={!View.hasNext(viewState.id)}
        onmousedown={(e: MouseEvent) => e.preventDefault()}
        onclick={(e: MouseEvent) => {
          e.preventDefault();
          View.forward(viewState);
        }}
      >
        <ChevronRight
          class={headerIconStyle}
          data-disabled={!View.hasNext(viewState.id)}
        />
      </Button>
      <Button class={headerButtonStyle}>
        <Search class={headerIconStyle} />
      </Button>
    </div>
    <div class={headerTitleContainer}>
      <Button class={headerTitleButtonStyle}>
        <CardStack class={headerTitleIconStyle} />
      </Button>
      <div class={headerTitleTextStyle}>
        {#if viewState.title.length === 0}
          Untitled
        {:else}
          {viewState.title}
        {/if}
      </div>
    </div>
    <div class={headerRightButtons}>
      {#if !pinned}
        <Button class={headerButtonStyle} onclick={onCloseButtonClick}>
          <X class={headerIconStyle} />
        </Button>
      {/if}
    </div>
  </div>

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
        bind:focusPosition={viewState.focusPosition}
        variant={{ style: "cardsViewTitle" }}
      />
    </div>
    <div class={paragraphContainerStyle}>
      <VerticalLine class={paragraphContainerLine} />
      {#each outline.paragraphs as paragraph (paragraph.id)}
        <Editor
          doc={paragraph}
          variant={{ style: "card" }}
          isViewFocused={isFocused}
          bind:focusPosition={viewState.focusPosition}
        />
      {/each}
    </div>
    <div class={contentBoxBottomSpace}>
      <!-- <div class={contetBoxBottomLine}></div> -->
      <VerticalLineWithCircle class={contetBoxBottomLine} />
    </div>
  </div>
</ScrollArea>

<script module>
  const headerStyle = css({
    position: "sticky",
    top: "0",
    zIndex: "local.header",
    display: "grid",
    gridTemplateColumns: "[1fr auto 1fr]",
    gap: "4",
    justifyContent: "space-between",
    alignItems: "center",
    w: "full",
    overflow: "hidden",
    h: "8",
    p: "1",
    bg: "view.bg/90",
    roundedTop: "md",
    backdropFilter: "[blur(4px)]",
  });

  const headerLeftButtons = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    gap: "2",
  });

  const headerRightButtons = css({
    flexBasis: "[auto]",
    flexGrow: "0",
    flexShrink: "0",
    display: "flex",
    flexDir: "row",
    justifyContent: "end",
    gap: "2",
  });

  const headerButtonStyle = css({
    justifySelf: "end",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    shadow: "[none]",
    bg: "transparent",
    _disabled: {
      _hover: {
        bg: "transparent",
      },
    },
    _hover: {
      bg: "view.bg-selected",
    },
  });

  const headerIconStyle = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
    "&[data-disabled=true]": {
      color: "view.text-muted/50",
    },
  });

  const headerTitleContainer = css({
    justifySelf: "center",
    w: "full",
    maxW: "[38.25rem]",
    flex: "[0 1 auto]",
    display: "flex",
    flexDir: "row",
    justifyContent: "center",
    alignItems: "center",
    h: "full",
    minW: "0",
    overflow: "hidden",
    textOverflow: "ellipsis",
  });

  const headerTitleButtonStyle = css({
    flexShrink: "0",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    transition: "colors",
    p: "0",
    w: "6",
    h: "6",
    rounded: "circle",
    bg: "transparent",
    _hover: {
      bg: "view.bg-selected",
    },
  });

  const headerTitleIconStyle = css({
    w: "3",
    h: "3",
    color: "view.text-muted",
  });

  const headerTitleTextStyle = css({
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
    color: "view.text-muted",
    fontSize: "xs",
    cursor: "default",
  });

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
  //
  // const titleOutlineEditorContainer = css.raw({
  //   w: "full",
  //   h: "fit",
  //   wordBreak: "break-word",
  //   minHeight: "[3rem]",
  //   color: "view.text",
  // });
</script>
