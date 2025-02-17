<script lang="ts">
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import CreateNewButton from "$lib/components/workspace/CreateNewButton.svelte";
  import { css } from "styled-system/css";
</script>

<div class={containerStyle}>
  <Sidebar />
  <div
    class={css({
      bg: "view.bg",
      w: "full",
      h: "full",
      rounded: "md",
      shadow: "md.around",
    })}
  ></div>
  <div
    class={css({
      bg: "view.bg",
      w: "full",
      h: "full",
      rounded: "md",
      shadow: "md.around",
    })}
  ></div>
  <CreateNewButton />
</div>

<script module>
  const containerStyle = css({
    display: "flex",
    flexDir: "row",
    gap: "2",
    bg: "workspace.bg",
    w: "screen",
    h: "screen",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
  });
</script>
