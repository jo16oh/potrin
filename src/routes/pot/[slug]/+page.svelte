<script lang="ts">
  // import CreateButton from "$lib/components/CreateButton.svelte";
  // import Editor from "$lib/components/Editor.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { App } from "$lib/models/App.svelte";

  const app = App.state();
</script>

<Sidebar>
  <div class="h-6 w-full bg-black">
    <button class="h-full w-96 bg-white" onclick={() => console.log("click")}
      >click me</button
    >
  </div>
  <!-- <Editor /> -->
  <!-- <CreateButton /> -->
  {#each Object.values(app.pots) as pot_name}
    <div>{pot_name}</div>
  {/each}
</Sidebar>
