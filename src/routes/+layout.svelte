<script>
  import "../app.css";
  import TitleBarHandler from "$lib/components/TitleBarHandler.svelte";
  import { commands } from "../generated/tauri-commands";

  const appState = commands.getAppState();
</script>

<TitleBarHandler />

{#await appState}
  loading...
{:then}
  <slot />
{/await}
