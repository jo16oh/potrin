<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import { Editor } from "@tiptap/core";
  import { onMount, onDestroy } from "svelte";
  import { createSearchQueryExtensions } from "$lib/components/editor/schema";
  import type { View } from "$lib/models/Workspace.svelte";

  type Props = { view: View<"search"> };

  let { view }: Props = $props();

  let search = new Search(view);
  let queryElement: HTMLDivElement;
  let queryEditor: Editor;

  onMount(() => {
    queryEditor = new Editor({
      element: queryElement,
      extensions: createSearchQueryExtensions(),
      content: search.query,
      onTransaction: () => (view.query = search.query = queryEditor.getText()),
    });
  });

  onDestroy(() => {
    queryEditor.destroy();
    search.cleanup();
  });
</script>

<div bind:this={queryElement}></div>

{#await search.result then result}
  {#each result as item}
    {item.outline.text}
    {#each item.paragraphs as paragraph}
      {JSON.stringify(paragraph.doc)}
    {/each}
  {/each}
{/await}

<script module>
</script>
