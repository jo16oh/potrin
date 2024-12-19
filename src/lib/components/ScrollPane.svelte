<script lang="ts" generics="T">
  import { tick } from "svelte";
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";

  type Props = {
    items: T[];
    maxLength: number;
    class: string;
    item: Snippet<[T]>;
    onReachTop?: (
      doUpdate: (exec: () => void) => Promise<void>,
    ) => Promise<void> | void;
    onReachBottom?: (
      doUpdate: (exec: () => void) => void,
    ) => Promise<void> | void;
    loading?: Snippet;
    header?: Snippet;
    footer?: Snippet;
  };

  let {
    items = $bindable(),
    maxLength,
    onReachTop,
    onReachBottom,
    class: className,
    item,
    loading,
    header,
    footer,
  }: Props = $props();

  let elem: HTMLElement;

  export function resetScroll() {
    elem.scrollTo({ top: 0, behavior: "instant" });
  }

  async function updateArrayHead(updateArray: () => void) {
    await tick();
    const scrollTopBefore = elem.scrollTop;
    const scrollHeightBefore = elem.scrollHeight;

    updateArray();
    await tick();

    const scrollHeightAfter = elem.scrollHeight;

    elem.scrollTo({
      top: scrollTopBefore + (scrollHeightAfter - scrollHeightBefore),
      behavior: "instant",
    });
  }

  let loadingTop: boolean = $state(false);
  let loadingBottom: boolean = $state(false);

  async function onscroll() {
    if (elem.scrollTop === 0) {
      if (!loadingTop) {
        loadingTop = true;

        const lengthBefore = items.length;
        await onReachTop?.(updateArrayHead);
        await tick();
        const lengthAfter = items.length;

        if (lengthAfter > maxLength) {
          const rowsAdded = lengthAfter - lengthBefore;
          if (rowsAdded > 0) {
            items = items.toSpliced(-rowsAdded);
          }
        }

        loadingTop = false;
      }
    } else if (elem.scrollHeight - elem.scrollTop <= elem.clientHeight) {
      if (!loadingBottom) {
        loadingBottom = true;

        const lengthBefore = items.length;
        await onReachBottom?.((update) => update());
        await tick();
        const lengthAfter = items.length;

        if (lengthAfter > maxLength) {
          const rowsAdded = lengthAfter - lengthBefore;
          if (rowsAdded > 0) {
            await updateArrayHead(() => {
              items = items.toSpliced(0, rowsAdded);
            });
          }
        }

        loadingBottom = false;
      }
    }
  }

  onMount(async () => {
    await onReachTop?.(updateArrayHead);
  });
</script>

<div style:position="relative">
  <div style="position: absolute; top: 0;">
    {#if loadingTop && loading}
      {@render loading()}
    {/if}
  </div>
  <div class={"scrollable-list " + className} bind:this={elem} {onscroll}>
    {#if header}
      {@render header()}
    {/if}
    {#each items as i, idx (idx)}
      {@render item(i)}
    {/each}
    {#if footer}
      {@render footer()}
    {/if}
  </div>
  <div style="position: absolute; bottom: 0;">
    {#if loadingBottom && loading}
      {@render loading()}
    {/if}
  </div>
</div>

<style>
  .scrollable-list {
    overflow-y: auto;
    scrollbar-width: none;
    -ms-overflow-style: none;
    scrollbar-width: none;
    scroll-behavior: smooth;
    overscroll-behavior: none;
  }
  .scrollable-list::-webkit-scrollbar {
    display: none;
  }
</style>
