<script lang="ts">
  import { commands, type ViewState } from "../../../generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import type { FocusPosition } from "../editor/utils";
  import CardsViewInner from "./CardsViewInner.svelte";

  type CardsViewState = Extract<ViewState, { type: "cards" }>;
  type Props = { viewState: CardsViewState };

  let { viewState = $bindable() }: Props = $props();

  let focusPosition: FocusPosition = $state.raw({ id: null, position: null });

  const promise = (async () => {
    const outlineId = viewState.id;
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
          focusPosition = { id: paragraph.id, position: "start" };
          return outline;
        })();

    return outline;
  })();
</script>

{#await promise then outline}
  <CardsViewInner {outline} bind:viewState bind:focusPosition />
{/await}
