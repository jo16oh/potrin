<script lang="ts">
  import { App } from "$lib/models/App.svelte";
  import { commands } from "../generated/tauri-commands";
  import * as Dialog from "$lib/components/ui/dialog";
  import Button from "$lib/components/ui/button/button.svelte";
  import { Input } from "$lib/components/ui/input";
  import { uuidv7 } from "$lib/utils";
  import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";

  const app = App.state();
  let pots = $state(commands.fetchPots());
  let dialogOpen = $state(false);
  let newPotName: string = $state("");

  async function createPot() {
    const pot = {
      id: uuidv7(),
      name: newPotName,
      owner: null,
    };
    await commands.createPot(pot);
    await app.changePot(pot.id);
    pots = commands.fetchPots();
    newPotName = "";
    dialogOpen = false;
  }

  function onkeypress(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.isComposing) {
      void createPot();
    }
  }

  async function selectPot(id: string) {
    await app.changePot(id);
  }
</script>

<div class="mx-auto mt-24 flex max-w-96 flex-col items-center justify-center">
  <img
    src="icon.svg"
    alt="Potrin logo"
    class="pointer-events-none h-48 w-48 drop-shadow-xl"
  />
  <img
    src="Potrin.svg"
    alt="Potrin logo"
    class="pointer-events-none w-40 py-2"
  />
  <div class="h-32"></div>

  <div class="flex w-full justify-between border-b px-2 py-2">
    <h2 class="text-md my-auto cursor-default select-none font-bold">
      Create new Pot
    </h2>
    <Dialog.Root bind:open={dialogOpen}>
      <Dialog.Trigger>
        <Button class="mr-1" variant="default">Create</Button>
      </Dialog.Trigger>
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Create new Pot</Dialog.Title>
          <Dialog.Description>
            <div class="flex flex-wrap justify-end">
              <Input
                class="my-4"
                bind:value={newPotName}
                placeholder="name"
                {onkeypress}
              />
              <Button class="ml-auto" onclick={createPot}>Create</Button>
            </div>
          </Dialog.Description>
        </Dialog.Header>
      </Dialog.Content>
    </Dialog.Root>
  </div>

  {#await pots then pots}
    {#if pots.length !== 0}
      <div class="w-full border-b py-4 pl-2">
        <h2 class="text-md my-auto cursor-default select-none font-bold">
          Select Pot
        </h2>
        <ScrollArea class="h-80 w-full">
          {#each pots as pot}
            <div
              class="my-1 mr-2 flex w-full justify-between rounded-sm py-1 pr-3 transition-all hover:bg-secondary"
            >
              <p
                class="text-md my-auto ml-2 cursor-default select-none font-semibold"
              >
                {pot.name}
              </p>
              <Button
                class="select-none"
                variant="outline"
                onclick={() => selectPot(pot.id)}>Open</Button
              >
            </div>
          {/each}
        </ScrollArea>
      </div>
    {/if}
  {/await}
</div>

