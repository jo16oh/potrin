<script lang="ts">
  import { ChevronRight, ClockArrowDown, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "../common/Button.svelte";
  import type { View } from "$lib/models/Workspace.svelte";
  import VirtualScroll from "../common/VirtualScroll.svelte";
  import { Timeline } from "$lib/models/Timeline.svelte";
  import { format } from "date-fns/format";
  import Asterisk from "../icon/Asterisk.svelte";
  import MockParagraphEditor from "../editor/MockParagraphEditor.svelte";
  import { onDestroy, onMount } from "svelte";
  import VerticalLineDash from "../icon/VerticalLineDash.svelte";
  import VerticalLine from "../icon/VerticalLine.svelte";
  import Tilda from "../icon/Tilda.svelte";
  import { debounce } from "es-toolkit";

  type Props = {
    timeline: Timeline;
    view: Extract<View, { type: "timeline" }>;
    pinned: boolean;
  };

  let { timeline, view, pinned }: Props = $props();

  let loading = $state(true);
  let virtualScrollRef = $state<ReturnType<typeof VirtualScroll>>()!;
  let scrollAreaRef = $state<HTMLDivElement>()!;
  let daysRef = $state<HTMLDivElement>()!;

  onMount(async () => {
    const dayStart = view.position?.dayStart;
    const scrollOffset = view.position?.scrollOffset;

    while (scrollAreaRef.scrollHeight <= scrollAreaRef.clientHeight) {
      const update = await timeline.loadBottom();
      if (update) {
        update();
      } else {
        break;
      }
    }

    if (dayStart && scrollOffset) {
      for (const e of daysRef.children) {
        if (
          Number(e.getAttribute("data-date")) === dayStart &&
          e instanceof HTMLElement
        ) {
          const scrollAmount = e.offsetTop + scrollOffset;

          while (
            scrollAreaRef.scrollHeight <=
            scrollAmount + scrollAreaRef.clientHeight
          ) {
            const update = await timeline.loadBottom();
            if (update) {
              update();
            } else {
              break;
            }
          }

          scrollAreaRef.scrollTo({
            top: scrollAmount,
          });

          break;
        }
      }
    }

    const update = await timeline.loadTop();
    if (update) await virtualScrollRef.updateArrayHead(update);
    loading = false;
  });

  onDestroy(timeline.cleanup);

  const onscroll = debounce(() => {
    const containerRect = scrollAreaRef.getBoundingClientRect();
    const elements = Array.from(daysRef.children);
    elements.sort(
      (a, b) =>
        Number(b.getAttribute("data-date")) -
        Number(a.getAttribute("data-date")),
    );

    for (const e of elements) {
      const eRect = e.getBoundingClientRect();
      const isVisible =
        eRect.bottom > containerRect.top && eRect.top < containerRect.bottom;
      const dayStart = Number(e.getAttribute("data-date"));

      if (isNaN(dayStart)) throw new Error("invalid date");

      if (isVisible) {
        view.position = {
          dayStart: dayStart,
          scrollOffset: containerRect.top - eRect.top,
        };
        break;
      }
    }
  }, 400);
</script>

<VirtualScroll
  bind:this={virtualScrollRef}
  bind:ref={scrollAreaRef}
  items={timeline.days}
  maxLength={28}
  onReachTop={async (doUpdate) => {
    for (const _ of Array.from(Array(7))) {
      const update = await timeline.loadTop();
      if (update) {
        await doUpdate(update);
      } else {
        break;
      }
    }
  }}
  onReachBottom={async (doUpdate) => {
    for (const _ of Array.from(Array(7))) {
      const update = await timeline.loadBottom();
      if (update) {
        doUpdate(update);
      } else {
        break;
      }
    }
  }}
  {onscroll}
>
  <div class={headerStyle}>
    <div class={headerLeftButtons}></div>
    <div class={headerTitleContainer}>
      <Button class={headerTitleButtonStyle}>
        <ClockArrowDown class={headerTitleIconStyle} />
      </Button>
      <div class={headerTitleTextStyle}>Timeline</div>
    </div>
    <div class={headerRightButtons}>
      {#if !pinned}
        <Button class={headerButtonStyle}>
          <X class={headerIconStyle} />
        </Button>
      {/if}
    </div>
  </div>

  <div bind:this={daysRef} class={contentContainerStyle} data-loading={loading}>
    {#each timeline.days as day (day.dayStart)}
      <div class="day" data-date={day.dayStart.getTime()}>
        <div class={dateStyle}>
          {format(day.dayStart, "yyyy-MM-dd")}
        </div>
        <div class={dayContentsStyle}>
          {#each day.items as { outline, paragraphs }}
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

              <div class={paragraphContainerStyle}>
                {#each paragraphs as paragraph, idx (paragraph.id)}
                  <div class={paragraphStyle}>
                    {#if paragraph.id in day.paragraphPositionIndex}
                      {@const index = day.paragraphPositionIndex[paragraph.id]}
                      {#if index && (index.prevId === paragraphs[idx - 1]?.id || index.prevId === null)}
                        <VerticalLine class={paragraphContainerLineTop} />
                      {:else}
                        <VerticalLineDash class={paragraphContainerLineTop} />
                        <Tilda class={tildaTop} />
                      {/if}
                    {/if}
                    <MockParagraphEditor
                      {paragraph}
                      variant={{ style: "card" }}
                    />
                    {#if idx === paragraphs.length - 1}
                      {@const index = day.paragraphPositionIndex[paragraph.id]}
                      {#if index?.isLast}
                        <VerticalLine class={paragraphContainerLineBottom} />
                        <div class={roundedLineEnd}></div>
                      {:else}
                        <VerticalLineDash
                          class={paragraphContainerLineBottom}
                        />
                        <Tilda class={tildaBottom} />
                      {/if}
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</VirtualScroll>

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
    "&[data-loading=true]": {
      opacity: "0",
    },
  });

  const dateStyle = css({
    fontSize: "4xl",
    fontWeight: "semibold",
  });

  const dayContentsStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    py: "6",
    gap: "4",
  });

  const outlineContainerStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    py: "2",
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
    position: "absolute",
    z: "-10",
    top: "-4",
    left: "[0.75rem]",
    color: "view.text-muted",
  });

  const paragraphContainerLineBottom = css({
    h: "4",
    position: "absolute",
    z: "-10",
    bottom: "-4",
    left: "[0.75rem]",
    color: "view.text-muted",
  });

  const paragraphStyle = css({
    w: "full",
    h: "fit",
    position: "relative",
  });

  const tildaTop = css({
    h: "2.5",
    w: "2.5",
    position: "absolute",
    z: "-10",
    top: "-3",
    left: "[0.5rem]",
    color: "view.text-muted",
  });

  const tildaBottom = css({
    h: "2.5",
    w: "2.5",
    position: "absolute",
    z: "-10",
    bottom: "-3.5",
    left: "[0.5rem]",
    color: "view.text-muted",
  });

  const roundedLineEnd = css({
    w: "1",
    h: "1",
    bg: "view.text-muted",
    rounded: "circle",
    position: "absolute",
    bottom: "-4",
    left: "[0.65rem]",
  });
</script>
