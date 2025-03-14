<script lang="ts">
  import { Outline } from "$lib/models/Outline.svelte";
  import { Paragraph } from "$lib/models/Paragraph.svelte";
  import CardsViewInner from "./CardsViewInner.svelte";
  import { css } from "styled-system/css";
  import { fetchTree } from "$lib/commands";
  import { watch } from "runed";
  import { View } from "$lib/models/Workspace.svelte";
  import Header from "../common/Header.svelte";
  import ViewHistoryNavigation from "../common/ViewHistoryNavigation.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import { Search, X } from "lucide-svelte";
  import CardStack from "$lib/components/icon/CardStack.svelte";

  type Props = {
    view: View<"cards">;
    isFocused: boolean;
    pinned?: boolean;
    onCloseButtonClick: () => void;
  };

  let {
    view = $bindable(),
    isFocused,
    pinned = false,
    onCloseButtonClick,
  }: Props = $props();

  let promise = $state<Promise<Outline>>();

  watch(
    () => view.outlineId,
    () => {
      if (view.outlineId) {
        if (promise) {
          promise.then(async (o) => {
            await View.save(view);

            if (view.outlineId && o.id !== view.outlineId) {
              fetchTree(view.outlineId, 2).then((o) => {
                promise = Promise.resolve(o);
              });
            }
          });
        } else {
          promise = fetchTree(view.outlineId, 2);
        }
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
  <Header>
    {#snippet left({ buttonStyle, iconStyle })}
      <ViewHistoryNavigation {view} {buttonStyle} {iconStyle} />
      <Button class={css(buttonStyle)}>
        <Search
          class={css(iconStyle)}
          onmousedown={(e: MouseEvent) => e.preventDefault()}
          onclick={(e: MouseEvent) => {
            e.preventDefault();
            View.open(view, { ...View.new("search"), scope: view.outlineId });
          }}
        />
      </Button>
    {/snippet}
    {#snippet center({ buttonStyle, iconStyle, textStyle })}
      <Button class={css(buttonStyle)}>
        <CardStack class={css(iconStyle)} />
      </Button>
      <div class={css(textStyle)}>
        {#if view.title.length}
          {view.title}
        {/if}
      </div>
    {/snippet}
    {#snippet right({ buttonStyle, iconStyle })}
      {#if !pinned}
        <Button class={css(buttonStyle)} onclick={onCloseButtonClick}>
          <X class={css(iconStyle)} />
        </Button>
      {/if}
    {/snippet}
  </Header>
  {#if promise}
    {#await promise then outline}
      <CardsViewInner {outline} bind:view {isFocused} />
    {/await}
  {/if}
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
    userSelect: "none",
  });
</script>
