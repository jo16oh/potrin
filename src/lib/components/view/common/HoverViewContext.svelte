<script lang="ts">
  import { css } from "styled-system/css";
  import { View, Workspace } from "$lib/models/Workspace.svelte";
  import { getContext, setContext, type Snippet } from "svelte";
  import type { DialogRootProps } from "bits-ui";
  import { HoverView } from "$lib/components/common/HoverView";
  import Button from "$lib/components/common/Button.svelte";
  import { Columns2, Link, Maximize2 } from "lucide-svelte";

  type AllowedViewType = "cards" | "outline";

  type Props = {
    children?: Snippet;
  } & DialogRootProps;

  let { children, ...restProps }: Props = $props();

  let workspaceState = Workspace.current.state;

  HoverViewContext.init();
  let context = HoverViewContext.state!;

  async function handleClickMaximize(e: MouseEvent) {
    // prevents editor from being blurred
    e.preventDefault();
    context.open = false;

    await View.save(context.view);

    setTimeout(() => {
      const newTabId = crypto.randomUUID();
      const newViewId = crypto.randomUUID();
      workspaceState.focusedTabId = newTabId;
      workspaceState.tabs.unshift({
        id: newTabId,
        views: [{ ...$state.snapshot(context.view), id: newViewId }],
        focusedViewId: newViewId,
      });
    });
  }
</script>

<HoverView.Context
  bind:view={context.view}
  bind:open={context.open}
  {...restProps}
>
  {@render children?.()}

  {#snippet rightsideTopButtons()}
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
      }}
      onclick={handleClickMaximize}
    >
      <Maximize2 class={iconInsideRightSideButton} />
    </Button>
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      <Columns2 class={iconInsideRightSideButton} />
    </Button>
    <Button
      class={rightSideButtonStyle}
      onmousedown={(e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      <Link class={iconInsideRightSideButton} />
    </Button>
  {/snippet}
</HoverView.Context>

<script module lang="ts">
  export class HoverViewContext {
    private static KEY = Symbol();

    view: View<AllowedViewType> = $state(View.new("cards"));
    open = $state(false);

    static init() {
      setContext(this.KEY, new HoverViewContext());
    }

    static get state() {
      return getContext<HoverViewContext | undefined>(this.KEY);
    }
  }

  const rightSideButtonStyle = css({
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    gap: "2",
    shadow: "sm",
    transition: "colors",
    p: "0",
    w: "8",
    h: "8",
    bg: "view.bg",
    _hover: {
      bg: "view.bg-selected",
    },
    rounded: "circle",
  });

  const iconInsideRightSideButton = css({
    w: "4",
    h: "4",
    color: "view.text-muted",
  });
</script>
