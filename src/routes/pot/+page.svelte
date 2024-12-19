<script lang="ts">
  import { App } from "$lib/models/App.svelte";
  import { type AppState } from "../../generated/tauri-commands";
  import CreateButton from "$lib/components/CreateButton.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";

  type Props = {
    initialState: AppState;
  };

  const { initialState }: Props = $props();

  App.init(initialState);

  const app = App.state();
</script>

{#if app.pot}
  <Sidebar
    sidebar={app.pot.workspace.sidebar}
    bind:workspace={app.pot.workspace}
  >
    <Editor />
    <CreateButton />
  </Sidebar>
{/if}
