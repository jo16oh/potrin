<script lang="ts">
  const dragRegionDiv = document.createElement("div");
  dragRegionDiv.setAttribute("data-tauri-drag-region", "");
  dragRegionDiv.className = "dragble-state";
  document.documentElement.insertBefore(dragRegionDiv, document.body);
</script>

<style>
  :global {
    .dragble-state {
      width: 100%;
      height: 28px;
      position: fixed;
      user-select: none;
      display: flex;
      justify-content: flex-end;
      top: 0;
      left: 0;
      z-index: 100;
    }
  }
</style>
