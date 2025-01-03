<script lang="ts">
  import * as Sidebar from "./ui/sidebar/index";
  import SidebarContent from "./SidebarContent.svelte";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";
  import { Workspace } from "$lib/models/Workspace.svelte";

  let {
    children,
  }: {
    children?: Snippet;
  } = $props();

  const workspace = Workspace.state();
  const sidebar = $derived(workspace.sidebar);

  let open = $state(workspace.sidebar.isFloat ? false : true);
  let width = $state(workspace.sidebar.width);
  $effect(() => {
    width = sidebar.width;
  });
  let isResizing = $state(false);

  function resize(e: MouseEvent) {
    isResizing = true;
    let prevX = e.clientX;
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    function handleMouseMove(e: MouseEvent) {
      if (!isResizing) return;

      const currentX = e.clientX;
      const currentWidth = width + (currentX - prevX);

      if (150 < currentWidth && currentWidth < 600) {
        width = currentWidth;
        prevX = currentX;
      }
    }

    function handleMouseUp() {
      isResizing = false;
      sidebar.resize(width);
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    }
  }

  function onmouseleave(
    e: MouseEvent & {
      currentTarget: EventTarget & HTMLDivElement;
    },
  ) {
    if (!sidebar.isFloat) return;

    const currentTarget = e.currentTarget;
    const currentTargetRect = currentTarget.getBoundingClientRect();

    const mouseX = e.clientX;
    const mouseY = e.clientY;

    const isInside =
      mouseX >= currentTargetRect.left &&
      mouseX <= currentTargetRect.right &&
      mouseY >= currentTargetRect.top &&
      mouseY <= currentTargetRect.bottom;

    // To prevent floating sidebar from closing when a selectable button is clicked
    if (!isInside && !isResizing) {
      open = false;
    } else {
      // To ignore the first click event that occurs when a MenuButton inside the sidebar is clicked
      setTimeout(() => {
        window.addEventListener("click", handleClick);
      }, 0);
    }

    function handleClick(e: MouseEvent) {
      const mouseX = e.clientX;
      const mouseY = e.clientY;

      const isInside =
        //@ts-expect-error check if the clicked element is a descendant of the sidebar element
        (e.target && currentTarget.contains(e.target)) ||
        // otherwise check if the clicked position is inside the sidebar
        (mouseX >= currentTargetRect.left &&
          mouseX <= currentTargetRect.right &&
          mouseY >= currentTargetRect.top &&
          mouseY <= currentTargetRect.bottom);

      if (!isInside && sidebar.isFloat) {
        open = false;
        window.removeEventListener("click", handleClick);
      }
    }
  }
</script>

{#if sidebar.isFloat}
  <div
    class="fixed h-screen w-6"
    onmouseenter={() => {
      open = true;
    }}
    role="presentation"
  ></div>
{/if}

<Sidebar.Provider
  class={cn(
    "select-none overflow-hidden overscroll-contain",
    isResizing ? "transition-none" : "",
  )}
  bind:open
  style={`--sidebar-width: ${width.toFixed()}px`}
>
  <Sidebar.Root
    variant={sidebar.isFloat ? "floating" : "sidebar"}
    class={cn(
      "bg-sidebar",
      sidebar.isFloat
        ? "fixed top-[1.25rem] h-[calc(100vh_-_1.25rem)] bg-transparent duration-150 ease-in-out"
        : "px-2 pb-2 transition-none",
      isResizing ? "transition-none" : "",
    )}
    {onmouseleave}
  >
    {#if !sidebar.isFloat}
      <div class="h-7 w-full"></div>
    {/if}
    <SidebarContent />
    <div
      onmousedown={resize}
      class={cn(
        sidebar.isFloat
          ? "right-[calc(0.375rem_+_1px)] top-4 h-[calc(100vh_-_3.25rem)]"
          : "right-[-1px] h-screen",
        "absolute w-0.5 cursor-col-resize bg-transparent transition-colors duration-100 hover:bg-sidebar-ring",
      )}
      role="presentation"
    ></div>
  </Sidebar.Root>

  <div class="h-screen w-full overflow-hidden">
    {@render children?.()}
  </div>
</Sidebar.Provider>
