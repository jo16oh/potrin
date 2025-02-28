<script lang="ts">
  import { type ViewState } from "../../../generated/tauri-commands";
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import CardsViewInner from "./CardsViewInner.svelte";
  import { css } from "styled-system/css";
  import { fetchTree } from "$lib/commands";
  import { watch } from "runed";
  import { View } from "$lib/models/Workspace.svelte";

  type CardsViewState = Extract<ViewState, { type: "cards" }>;
  type Props = {
    viewState: CardsViewState;
    isFocused: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    viewState: view = $bindable(),
    isFocused,
    onCloseButtonClick,
  }: Props = $props();

  let promise = $state(
    view.outlineId
      ? fetchTree(view.outlineId, 2)
      : createNewOutline().then((o) => {
          view.focusPosition.id = o.id;
          return o;
        }),
  );

  watch(
    () => view.outlineId,
    () => {
      if (view.outlineId) {
        promise.then(async (o) => {
          await View.save(view);

          if (view.outlineId && o.id !== view.outlineId) {
            fetchTree(view.outlineId, 2).then((o) => {
              promise = Promise.resolve(o);
            });
          }
        });
      } else {
        (async () => {
          await View.save(view);

          createNewOutline().then((o) => {
            view.outlineId = o.id;
            promise = Promise.resolve(o);
          });
        })();
      }
    },
  );

  async function createNewOutline() {
    const outline = await Outline.new();
    const paragraph = Paragraph.new(outline);
    outline.insertParagraph(paragraph);
    view.focusPosition = { id: paragraph.id, position: "start" };
    return outline;
  }
</script>

<div class={viewContainer}>
  {#await promise then outline}
    <CardsViewInner
      {outline}
      bind:viewState={view}
      {isFocused}
      {onCloseButtonClick}
    />
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
    justifyContent: "center",
    alignItems: "center",
    shadow: "md.around",
    overflow: "hidden",
  });
</script>
