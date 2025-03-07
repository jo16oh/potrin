<script lang="ts">
  import { css } from "styled-system/css";
  import type { View } from "$lib/models/Workspace.svelte";
  import { Timeline } from "$lib/models/Timeline.svelte";
  import TimelineViewInner from "./TimelineViewInner.svelte";

  type Props = { view: Extract<View, { type: "timeline" }>; pinned: boolean };
  let { view, pinned }: Props = $props();

  let promise = $state(
    Timeline.init(view.position ? { at: view.position.dayStart } : "latest"),
  );
</script>

<div class={viewContainer}>
  {#await promise then timeline}
    <TimelineViewInner {timeline} {view} {pinned} />
  {/await}
</div>

<script module>
  const viewContainer = css({
    position: "relative",
    w: "full",
    h: "full",
    bg: "view.bg",
    rounded: "md",
    display: "flex",
    flexDir: "column",
    shadow: "md.around",
    overflow: "hidden",
  });
</script>
