<script lang="ts">
  import { Search } from "lucide-svelte";
  import { css } from "styled-system/css";
  import HoverView from "$lib/components/common/HoverView";
  import { View } from "$lib/models/Workspace.svelte";
  import { watch } from "runed";

  HoverView.State.init("search");
  const state = HoverView.State.current!;

  let query = "";

  watch(
    // @ts-expect-error property if it's SearchView
    () => state.view.query,
    () => {
      if (state.view.type === "search") {
        query = state.view.query;
      }
    },
  );
</script>

<HoverView.Context>
  <HoverView.Trigger
    class={floatingButtonStyle}
    onmousedown={(e) => {
      e.preventDefault();
    }}
    onclick={() => {
      state.view = { ...View.new("search"), query: query };
    }}
  >
    <Search class={floatingButtonIconStyle} />
  </HoverView.Trigger>
</HoverView.Context>

<script module>
  const floatingButtonStyle = css({
    zIndex: "global.float",
    position: "absolute",
    right: "[24px]",
    bottom: "[78px]",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    p: "0",
    w: "14",
    h: "14",
    bg: "workspace.bg/90",
    rounded: "circle",
    transition: "colors",
    _hover: {
      bg: "workspace.bg-selected",
    },
  });

  const floatingButtonIconStyle = css({
    w: "6",
    h: "6",
    color: "[white]",
  });
</script>
