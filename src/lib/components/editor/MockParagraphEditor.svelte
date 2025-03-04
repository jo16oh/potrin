<script lang="ts">
  import type { Paragraph } from "$lib/models/Paragraph.svelte";
  import { paragraphEditorStyle } from "./styles";
  import type { SvelteHTMLElements } from "svelte/elements";

  type Props = {
    paragraph: Paragraph;
    variant: Parameters<typeof paragraphEditorStyle>[0];
  } & SvelteHTMLElements["div"];

  let { paragraph, variant, ...restProps }: Props = $props();
</script>

<div
  class={paragraphEditorStyle(variant) + " tiptap ProseMirror"}
  {...restProps}
>
  {#each paragraph.doc.content ?? [] as content}
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
