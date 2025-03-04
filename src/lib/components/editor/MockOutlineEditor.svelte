<script lang="ts">
  import type { Outline } from "$lib/models/Outline.svelte";
  import { outlineEditorStyle } from "./styles";
  import type { SvelteHTMLElements } from "svelte/elements";

  type Props = {
    outline: Outline;
    variant: Parameters<typeof outlineEditorStyle>[0];
  } & SvelteHTMLElements["div"];

  let { outline, variant, ...restProps }: Props = $props();
</script>

<div class={outlineEditorStyle(variant) + " tiptap ProseMirror"} {...restProps}>
  {#each outline.doc.content ?? [] as content}
    {#if content.type === "paragraph"}
      <p>
        {#each content.content ?? [] as c}
          {#if c.type === "text"}
            {c.text}
          {/if}
        {/each}
      </p>
    {/if}
  {/each}
</div>
