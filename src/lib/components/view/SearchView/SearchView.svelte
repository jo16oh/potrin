<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import type { View } from "$lib/models/Workspace.svelte";
  import { css } from "styled-system/css";
  import SearchViewInner from "./SearchViewInner.svelte";

  type Props = {
    view: View<"search">;
    pinned: boolean;
    onCloseButtonClick: () => void;
  };

  let { view, pinned, onCloseButtonClick }: Props = $props();

  let search = Search.init(view);
</script>

<div class={viewContainer}>
  {#await search then search}
    <SearchViewInner {view} {search} {pinned} {onCloseButtonClick} />
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
