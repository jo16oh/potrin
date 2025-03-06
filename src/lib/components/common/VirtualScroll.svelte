<script lang="ts" generics="T">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";
  import ScrollArea from "./ScrollArea.svelte";
  import { css } from "styled-system/css";
  import type { SvelteHTMLElements, UIEventHandler } from "svelte/elements";

  type Props = {
    ref?: HTMLDivElement | undefined;
    items: T[];
    maxLength: number;
    onReachTop?: (
      doUpdate: (fn: () => void) => Promise<void>,
    ) => Promise<void> | void;
    onReachBottom?: (
      doUpdate: (fn: () => void) => void,
    ) => Promise<void> | void;
    loading?: Snippet;
    children?: Snippet;
  } & SvelteHTMLElements["div"];

  let {
    ref = $bindable(),
    items = $bindable(),
    maxLength,
    onReachTop,
    onReachBottom,
    loading,
    children,
    onscroll,
    ...restProps
  }: Props = $props();

  let loadingTop: boolean = $state(false);
  let loadingBottom: boolean = $state(false);

  export async function updateArrayHead(updateArray: () => void) {
    await tick();
    const scrollTopBefore = ref?.scrollTop ?? 0;
    const scrollHeightBefore = ref?.scrollHeight ?? 0;

    updateArray();
    await tick();

    const scrollHeightAfter = ref?.scrollHeight ?? 0;

    ref?.scrollTo({
      top: scrollTopBefore + (scrollHeightAfter - scrollHeightBefore),
      behavior: "instant",
    });
  }

  const handleScroll: UIEventHandler<HTMLDivElement> = async (e) => {
    const scrollTop = ref?.scrollTop ?? 0;
    const scrollHeight = ref?.scrollHeight ?? 0;
    const clientHeight = ref?.clientHeight ?? 0;

    if (scrollTop === 0) {
      if (!loadingTop) {
        loadingTop = true;

        const lengthBefore = items.length;
        await onReachTop?.(updateArrayHead);
        await tick();
        const lengthAfter = items.length;

        if (lengthAfter > maxLength) {
          const rowsAdded = lengthAfter - lengthBefore;
          if (rowsAdded > 0) {
            items.splice(-rowsAdded);
          }
        }

        loadingTop = false;
      }
    } else if (scrollHeight - scrollTop <= clientHeight) {
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
              items.splice(0, rowsAdded);
            });
          }
        }

        loadingBottom = false;
      }
    }

    onscroll?.(e);
  };
</script>

<ScrollArea
  bind:ref
  type="always"
  {scrollAreaStyle}
  onscroll={handleScroll}
  {...restProps}
>
  {#if loadingTop}
    <div class={loadingTopStyle}>
      {@render loading?.()}
    </div>
  {/if}
  {@render children?.()}
  {#if loadingBottom}
    <div class={loadingBottomStyle}>
      {@render loading?.()}
    </div>
  {/if}
</ScrollArea>

<script module>
  const scrollAreaStyle = css.raw({ position: "relative" });

  const loadingTopStyle = css({
    position: "sticky",
    top: "0",
  });

  const loadingBottomStyle = css({
    position: "sticky",
    bottom: "0",
  });
</script>
