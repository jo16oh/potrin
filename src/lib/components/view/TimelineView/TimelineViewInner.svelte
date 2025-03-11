<script lang="ts">
  import { ChevronRight, ClockArrowDown, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import VirtualScroll from "$lib/components/common/VirtualScroll.svelte";
  import { Timeline } from "$lib/models/Timeline.svelte";
  import { format } from "date-fns/format";
  import Asterisk from "$lib/components/icon/Asterisk.svelte";
  import { onDestroy, onMount } from "svelte";
  import VerticalLineDash from "$lib/components/icon/VerticalLineDash.svelte";
  import VerticalLine from "$lib/components/icon/VerticalLine.svelte";
  import Tilda from "$lib/components/icon/Tilda.svelte";
  import { debounce } from "es-toolkit";
  import HoverViewContext from "../common/HoverViewContext.svelte";
  import HoverViewTriggerParagraph from "../common/HoverViewTriggerParagraph.svelte";
  import VerticalLineWithCircle from "$lib/components/icon/VerticalLineWithCircle.svelte";
  import Header from "../common/Header.svelte";

  type Props = {
    timeline: Timeline;
    view: View<"timeline">;
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

    // load bottom until the timeline is scrollable
    while (scrollAreaRef.scrollHeight <= scrollAreaRef.clientHeight) {
      const update = await timeline.loadBottom();
      if (update) {
        update();
      } else {
        break;
      }
    }

    if (dayStart && scrollOffset) {
      const dayRef = document.getElementById(view.id + dayStart.toString());
      if (!dayRef) return;

      const scrollAmount = dayRef.offsetTop + scrollOffset;

      // load bottom until the timeline is fully scrollable to the previous position
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

      // scroll to previous position
      scrollAreaRef.scrollTo({
        top: scrollAmount,
      });
    }

    // load top
    const update = await timeline.loadTop();
    if (update) await virtualScrollRef.updateArrayHead(update);
    loading = false;
  });

  onDestroy(timeline.cleanup);

  async function handleHeaderClick() {
    await timeline.latest();
    scrollAreaRef.scrollTo({ top: 0 });

    // load bottom until the timeline is scrollable
    while (scrollAreaRef.scrollHeight <= scrollAreaRef.clientHeight) {
      const update = await timeline.loadBottom();
      if (update) {
        update();
      } else {
        break;
      }
    }
  }

  const onscroll = debounce(() => {
    const containerRect = scrollAreaRef.getBoundingClientRect();
    const elements = Array.from(daysRef.children);
    elements.sort(
      (a, b) =>
        Number(b.id.slice(view.id.length)) - Number(a.id.slice(view.id.length)),
    );

    for (const e of elements) {
      const eRect = e.getBoundingClientRect();
      const isVisible =
        eRect.bottom > containerRect.top && eRect.top < containerRect.bottom;
      const dayStart = Number(e.id.slice(view.id.length));

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

<Header onclick={handleHeaderClick}>
  {#snippet center({ buttonStyle, iconStyle, textStyle })}
    <Button class={css(buttonStyle)}>
      <ClockArrowDown class={css(iconStyle)} />
    </Button>
    <div class={css(textStyle)}>Timeline</div>
  {/snippet}
  {#snippet right({ buttonStyle, iconStyle })}
    {#if !pinned}
      <Button class={css(buttonStyle)}>
        <X class={css(iconStyle)} />
      </Button>
    {/if}
  {/snippet}
</Header>

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
  <HoverViewContext>
    <div
      bind:this={daysRef}
      class={contentContainerStyle}
      data-loading={loading}
    >
      {#each timeline.days as day (day.dayStart)}
        {#if day.items.length}
          <div class="day" id={view.id + day.dayStart.getTime().toString()}>
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
                          {@const index =
                            day.paragraphPositionIndex[paragraph.id]}
                          {#if index && (index.prevId === paragraphs[idx - 1]?.id || index.prevId === null)}
                            <VerticalLine class={paragraphContainerLineTop} />
                          {:else}
                            <VerticalLineDash
                              class={paragraphContainerLineTop}
                            />
                            <div class={tildaTopContainer}>
                              <Tilda class={tildaTop} />
                            </div>
                          {/if}
                        {/if}
                        <HoverViewTriggerParagraph {paragraph} />
                        {#if idx === paragraphs.length - 1}
                          {@const index =
                            day.paragraphPositionIndex[paragraph.id]}
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
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/each}
    </div>
  </HoverViewContext>
</VirtualScroll>

<script module>
  const contentContainerStyle = css({
    display: "flex",
    flexDir: "column",
    gap: "9",
    maxW: "[38.25rem]",
    px: "2",
    py: "32",
    m: "auto",
    "&[data-loading=true]": {
      opacity: "0",
    },
  });

  const dateStyle = css({
    fontSize: "3xl",
    fontWeight: "semibold",
  });

  const dayContentsStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    py: "4",
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
