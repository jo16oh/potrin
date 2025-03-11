<script lang="ts">
  import { Search } from "$lib/models/Search.svelte";
  import { Editor } from "@tiptap/core";
  import { onMount, onDestroy } from "svelte";
  import { createSearchQueryExtensions } from "$lib/components/editor/schema";
  import type { View } from "$lib/models/Workspace.svelte";
  import Header from "../common/Header.svelte";
  import { SearchIcon, X } from "lucide-svelte";
  import { css } from "styled-system/css";
  import Button from "$lib/components/common/Button.svelte";

  type Props = { view: View<"search">; pinned: boolean };

  let { view, pinned }: Props = $props();

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

<Header>
  {#snippet right({ buttonStyle, iconStyle })}
    {#if !pinned}
      <Button class={css(buttonStyle)}>
        <X class={css(iconStyle)} />
      </Button>
    {/if}
  {/snippet}
  {#snippet center({ buttonStyle, iconStyle, textStyle })}
    <Button class={css(buttonStyle)}>
      <SearchIcon class={css(iconStyle)} />
    </Button>
    <div class={css(textStyle)}>Search</div>
  {/snippet}
</Header>

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
