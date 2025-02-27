<script lang="ts">
  import { ChevronLeft, ChevronRight, Search, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "../common/Button.svelte";
  import OutlineEditor from "../editor/OutlineEditor.svelte";
  import { type ViewState } from "../../../generated/tauri-commands";
  import { Outline } from "$lib/models/Outline.svelte";
  import Asterisk from "../icon/Asterisk.svelte";
  import ScrollArea from "../common/ScrollArea.svelte";
  import ParagraphEditor from "../editor/ParagraphEditor.svelte";
  import { debounce } from "es-toolkit";
  import { watch } from "runed";
  import { onMount } from "svelte";
  import CardStack from "../icon/CardStack.svelte";

  type CardsViewState = Extract<ViewState, { type: "cards" }>;

  type Props = {
    outline: Outline;
    viewState: CardsViewState;
    isFocused: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    outline,
    viewState = $bindable(),
    isFocused,
    onCloseButtonClick,
  }: Props = $props();

  let scrollAreaRef = $state<ReturnType<typeof ScrollArea> | undefined>();

  watch(
    () => outline.text,
    debounce(() => (viewState.title = outline.text), 16),
  );

  onMount(() => {
    scrollAreaRef?.scrollTo(0, viewState.scrollPosition);
  });

  const onscroll = debounce(() => {
    if (scrollAreaRef) viewState.scrollPosition = scrollAreaRef?.getScrollTop();
  }, 100);
</script>

<ScrollArea bind:this={scrollAreaRef} orientation="vertical" {onscroll}>
  <div class={headerStyle}>
    <div class={headerLeftButtons}>
      <Button class={headerButtonStyle} disabled={true}>
        <ChevronLeft class={headerIconStyle} />
      </Button>
      <Button class={headerButtonStyle} disabled={true}>
        <ChevronRight class={headerIconStyle} />
      </Button>
      <Button class={headerButtonStyle} disabled={true}>
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
      <Button class={headerButtonStyle} onclick={onCloseButtonClick}>
        <X class={headerIconStyle} />
      </Button>
    </div>
  </div>

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
        isViewFocused={isFocused}
        bind:focusPosition={viewState.focusPosition}
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
            isViewFocused={isFocused}
            bind:focusPosition={viewState.focusPosition}
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
  });

  const headerTitleContainer = css({
    justifySelf: "center",
    w: "full",
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
