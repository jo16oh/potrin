<script lang="ts">
  import "../app.css";
  import TitleBarHandler from "$lib/components/TitleBarHandler.svelte";
  import type { LayoutData } from "./$types";
  import { App } from "$lib/models/App.svelte";
  import { onMount, type Snippet } from "svelte";
  import { getCurrent } from "@tauri-apps/api/webviewWindow";

  let { data, children }: { data: LayoutData; children: Snippet } = $props();
  App.init(data.appState);

  onMount(async () => {
    await getCurrent().show();
  });

  window.addEventListener("keydown", (e) => {
    if (e.key === "w" && e.metaKey) {
      e.preventDefault();
      e.stopPropagation();
      console.log("cmd + w");
    } else if (e.key === "q" && e.metaKey) {
      // e.preventDefault();
      // e.stopPropagation();
      // console.log("cmd + q");
    }
  });
</script>

<TitleBarHandler />

{@render children()}
