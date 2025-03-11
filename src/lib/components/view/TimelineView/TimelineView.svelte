<script lang="ts">
  import { css } from "styled-system/css";
  import type { View } from "$lib/models/Workspace.svelte";
  import { Timeline } from "$lib/models/Timeline.svelte";
  import TimelineViewInner from "./TimelineViewInner.svelte";

  type Props = { view: View<"timeline">; pinned: boolean };
  let { view, pinned }: Props = $props();

  let promise = $state(
    Timeline.init(view.position ? { at: view.position.dayStart } : "latest"),
  );

  // import { Outline } from "$lib/models/Outline.svelte";
  // import { Paragraph } from "$lib/models/Paragraph.svelte";
  // import { subDays } from "date-fns";
  // import { range } from "es-toolkit";
  // (async () => {
  //   for (const i of range(100)) {
  //     const o = await Outline.new();
  //     const p = Paragraph.new(o);
  //
  //     o.createdAt = subDays(o.createdAt, i + 1);
  //     p.createdAt = subDays(p.createdAt, i + 1);
  //     o._doc = {
  //       type: "doc",
  //       content: [
  //         {
  //           type: "paragraph",
  //           content: [{ type: "text", text: "吾輩は猫である" }],
  //         },
  //       ],
  //     };
  //     p._doc = {
  //       type: "doc",
  //       content: [
  //         {
  //           type: "paragraph",
  //           content: [{ type: "text", text: "吾輩は猫である" }],
  //         },
  //       ],
  //     };
  //
  //     await o.save();
  //     await p.save();
  //   }
  // })();
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
