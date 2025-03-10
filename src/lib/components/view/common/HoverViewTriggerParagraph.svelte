<script lang="ts">
  import type { Paragraph } from "$lib/models/Paragraph.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import MockParagraphEditor from "$lib/components/editor/MockParagraphEditor.svelte";
  import { HoverView } from "$lib/components/common/HoverView";
  import { HoverViewContext } from "./HoverViewContext.svelte";
  import { css } from "styled-system/css";

  type AllowedViewType = "cards" | "outline";

  type Props = {
    paragraph: Paragraph;
  };

  let { paragraph }: Props = $props();

  let context = HoverViewContext.state;

  function createNewView(): View<AllowedViewType> {
    return {
      ...View.new("cards"),
      outlineId: paragraph.outlineId,
      focusPosition: { id: paragraph.id, position: "end" },
    };
  }
</script>

<HoverView.Trigger
  onclick={(e) => {
    if (window.getSelection()?.toString().length ?? 0 > 0) {
      e.preventDefault();
    } else {
      context.view = createNewView();
    }
  }}
>
  {#snippet child({ props })}
    <div class={triggerStyle} {...props}>
      <MockParagraphEditor {paragraph} variant={{ style: "card" }} />
    </div>
  {/snippet}
</HoverView.Trigger>

<script module>
  const triggerStyle = css({
    w: "full",
    h: "fit",
    textAlign: "start",
    _hover: {
      cursor: "pointer",
    },
  });
</script>
