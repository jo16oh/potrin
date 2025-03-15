<script lang="ts">
  import { ClockArrowDown, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import VirtualScroll from "$lib/components/common/VirtualScroll.svelte";
  import { Timeline } from "$lib/models/Timeline.svelte";
  import { format } from "date-fns/format";
  import { onDestroy, onMount } from "svelte";
  import { debounce } from "es-toolkit";
  import Header from "../common/Header.svelte";
  import FlattenDocList from "../common/FlattenDocList.svelte";
  import HoverView from "$lib/components/common/HoverView";

  type Props = {
    timeline: Timeline;
    view: View<"timeline">;
    pinned: boolean;
  };

  let { timeline, view, pinned }: Props = $props();

  if (pinned) HoverView.State.init();

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
  <div bind:this={daysRef} class={contentContainerStyle} data-loading={loading}>
    {#each timeline.days as day (day.dayStart)}
      {#if day.items.length}
        <div class="day" id={view.id + day.dayStart.getTime().toString()}>
          <div class={dateStyle}>
            {format(day.dayStart, "yyyy-MM-dd")}
          </div>
          {#if pinned}
            <HoverView.Context>
              <FlattenDocList
                {view}
                class={dayContentsStyle}
                items={day.items}
                paragraphPositionIndex={day.paragraphPositionIndex}
                openLinkOnHover={true}
              />
            </HoverView.Context>
          {:else}
            <FlattenDocList
              {view}
              class={dayContentsStyle}
              items={day.items}
              paragraphPositionIndex={day.paragraphPositionIndex}
              openLinkOnHover={false}
            />
          {/if}
        </div>
      {/if}
    {/each}
  </div>
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
    cursor: "default",
  });

  const dayContentsStyle = css({
    display: "flex",
    flexDir: "column",
    w: "full",
    h: "fit",
    py: "4",
    gap: "4",
  });
</script>
