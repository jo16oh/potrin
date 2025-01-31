<script lang="ts">
  import { App } from "$lib/models/App.svelte";
  import { css } from "styled-system/css";

  const [appState, updateAppState] = App.state();
</script>

<div class={css({ color: "button.text-muted" })}>Hello 🐼!</div>
