<script lang="ts">
  import { commands, type ViewState } from "../../../generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import CardsViewInner from "./CardsViewInner.svelte";
  import { css } from "styled-system/css";

  type CardsViewState = Extract<ViewState, { type: "cards" }>;
  type Props = {
    viewState: CardsViewState;
    isFocused: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    viewState = $bindable(),
    isFocused,
    onCloseButtonClick,
  }: Props = $props();

  const promise = (async () => {
    const outlineId = viewState.outlineId;
    const outline = outlineId
      ? await commands
          .fetchTree(outlineId, 2)
          .then(unwrap)
          .then(([outlines, paragraphs]) =>
            Outline.tree(outlines, paragraphs, outlineId),
          )
      : await (async () => {
          const outline = await Outline.new();
          const paragraph = Paragraph.new(outline);
          outline.insertParagraph(paragraph);
          viewState.outlineId = outline.id;
          viewState.focusPosition = { id: paragraph.id, position: "start" };
          return outline;
        })();

    return outline;
  })();
</script>

<div class={viewContainer}>
  {#await promise then outline}
    <CardsViewInner {outline} bind:viewState {isFocused} {onCloseButtonClick} />
  {/await}
</div>

<script module>
  const viewContainer = css({
    w: "full",
    h: "full",
    bg: "view.bg",
    rounded: "md",
    display: "flex",
    flexDir: "column",
    justifyContent: "center",
    alignItems: "center",
    shadow: "md.around",
  });
</script>
