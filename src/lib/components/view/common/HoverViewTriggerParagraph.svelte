<script lang="ts">
  import type { Paragraph } from "$lib/models/Paragraph.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import MockParagraphEditor from "$lib/components/editor/MockParagraphEditor.svelte";
  import { HoverView } from "$lib/components/common/HoverView";
  import { HoverViewContext } from "./HoverViewContext.svelte";
  import { css } from "styled-system/css";

  type AllowedView = Extract<View, { type: "cards" | "outline" }>;

  type Props = {
    paragraph: Paragraph;
  };

  let { paragraph }: Props = $props();

  let context = HoverViewContext.state;

  function createNewView(): AllowedView {
    return {
      ...View.new("cards"),
      outlineId: paragraph.outlineId,
      focusPosition: { id: paragraph.id, position: "end" },
    };
  }
</script>

<HoverView.Trigger
  class={triggerStyle}
  onmousedown={(e) => {
    e.preventDefault();
    context.view = createNewView();
  }}
>
  <MockParagraphEditor {paragraph} variant={{ style: "card" }} />
</HoverView.Trigger>

<script module>
  const triggerStyle = css({
    w: "full",
    h: "fit",
    textAlign: "start",
  });
</script>
