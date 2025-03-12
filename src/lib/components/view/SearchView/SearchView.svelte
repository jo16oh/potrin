<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import type { View } from "$lib/models/Workspace.svelte";
  import SearchViewInner from "./SearchViewInner.svelte";

  type Props = { view: View<"search">; pinned: boolean };

  let { view, pinned }: Props = $props();

  let search = Search.init(view);
</script>

{#await search then search}
  <SearchViewInner {view} {search} {pinned} />
{/await}
