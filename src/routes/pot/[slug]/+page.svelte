<script lang="ts">
  import TitleBarHandler from "$lib/components/common/TitleBarHandler.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import { css } from "styled-system/css";
</script>

<TitleBarHandler />

<div class={containerStyle}>
  <Sidebar />
</div>

<script module>
  const containerStyle = css({
    display: "flex",
    flexDir: "row",
    gap: "2",
    bg: "accent.bg",
    w: "screen",
    h: "screen",
    pt: "[28px]",
    pb: "[8px]",
    px: "[8px]",
  });
</script>
