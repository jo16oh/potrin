<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import type { View } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import SearchViewInner from "./SearchViewInner.svelte";
  import Header from "../common/Header.svelte";
  import ViewHistoryNavigation from "../common/ViewHistoryNavigation.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import { SearchIcon, X } from "lucide-svelte";

  type Props = {
    view: View<"search">;
    pinned: boolean;
    onCloseButtonClick: () => void;
  };

  let { view = $bindable(), pinned, onCloseButtonClick }: Props = $props();

  let search = Search.init(view);
</script>

<div class={viewContainer}>
  <Header>
    {#snippet left({ buttonStyle, iconStyle })}
      <ViewHistoryNavigation {view} {buttonStyle} {iconStyle} />
    {/snippet}
    {#snippet center({ buttonStyle, iconStyle, textStyle })}
      <Button class={css(buttonStyle)}>
        <SearchIcon class={css(iconStyle)} />
      </Button>
      <div class={css(textStyle)}>Search</div>
    {/snippet}
    {#snippet right({ buttonStyle, iconStyle })}
      {#if !pinned}
        <Button class={css(buttonStyle)} onclick={onCloseButtonClick}>
          <X class={css(iconStyle)} />
        </Button>
      {/if}
    {/snippet}
  </Header>

  {#await search then search}
    <SearchViewInner bind:view {search} {pinned} />
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
