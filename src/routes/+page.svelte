<script lang="ts">
  import { App } from "$lib/models/App.svelte";
  import { commands } from "../generated/tauri-commands";
  // import * as Dialog from "$lib/components/ui/dialog";
  // import Button from "$lib/components/ui/button/button.svelte";
  // import { Input } from "$lib/components/ui/input";
  import { uuidv7 } from "$lib/utils";
  // import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
  import { getCurrent } from "@tauri-apps/api/window";

  const app = App.state();
  let pots = $state(commands.fetchPots());
  let newPotDialogOpen = $state(false);
  let newPotName: string = $state("");
  let potSelectDialogOpen = $state(false);

  const version = commands.appVersion();

  async function createPot() {
    const pot = {
      id: uuidv7(),
      name: newPotName,
      owner: null,
      createdAt: new Date().getUTCMilliseconds(),
    };
    console.log(pot.id);
    await commands.createPot(pot);
    app.openPot(pot);
    pots = commands.fetchPots();
    newPotName = "";
    newPotDialogOpen = false;
  }

  function onkeypress(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.isComposing) {
      void createPot();
    }
  }

  function selectPot(pot: { id: string; name: string }) {
    app.openPot(pot);
    getCurrent().close();
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
  {#await version then v}
    <div class="p-2 text-sm text-secondary-foreground">Version {v}</div>
  {/await}

  <hr class="h-32" />

  <div class="flex w-full justify-between border-b px-2 py-2">
    <div class="text-md my-auto cursor-default select-none font-semibold">
      Create new pot
    </div>
    <!-- <Dialog.Root bind:open={newPotDialogOpen}> -->
    <!--   <Button -->
    <!--     class="mr-1" -->
    <!--     variant="default" -->
    <!--     onclick={() => (newPotDialogOpen = true)}>Create</Button -->
    <!--   > -->
    <!--   <Dialog.Content> -->
    <!--     <Dialog.Header> -->
    <!--       <Dialog.Title>Create new Pot</Dialog.Title> -->
    <!--       <Dialog.Description> -->
    <!--         <div class="flex flex-wrap justify-end"> -->
    <!--           <Input -->
    <!--             class="my-4" -->
    <!--             bind:value={newPotName} -->
    <!--             placeholder="name" -->
    <!--             {onkeypress} -->
    <!--           /> -->
    <!--           <Button class="ml-auto" onclick={createPot}>Create</Button> -->
    <!--         </div> -->
    <!--       </Dialog.Description> -->
    <!--     </Dialog.Header> -->
    <!--   </Dialog.Content> -->
    <!-- </Dialog.Root> -->
  </div>

  {#await pots then pots}
    {#if pots.length !== 0}
      <div class="flex w-full justify-between border-b px-2 py-2">
        <div class="text-md my-auto cursor-default select-none font-semibold">
          Open pot
        </div>
        <!-- <Dialog.Root bind:open={potSelectDialogOpen}> -->
        <!--   <Button -->
        <!--     class="mr-1" -->
        <!--     variant="default" -->
        <!--     onclick={() => (potSelectDialogOpen = true)}>Select</Button -->
        <!--   > -->
        <!--   <Dialog.Content> -->
        <!--     <Dialog.Header> -->
        <!--       <Dialog.Title>Open pot</Dialog.Title> -->
        <!--       <Dialog.Description> -->
        <!--         <ScrollArea class="h-80 w-full"> -->
        <!--           <ul> -->
        <!--             {#each pots as pot} -->
        <!--               {@const createdAt = new Date( -->
        <!--                 pot.createdAt, -->
        <!--               ).toLocaleDateString()} -->
        <!--               <li -->
        <!--                 class="flex w-full justify-between rounded-sm px-3 py-2 transition-all" -->
        <!--               > -->
        <!--                 <Button -->
        <!--                   class="flex w-full select-none border-none text-foreground" -->
        <!--                   variant="outline" -->
        <!--                   onclick={() => selectPot(pot)} -->
        <!--                 > -->
        <!--                   <div class="flex-grow"> -->
        <!--                     <div class="text-lg"> -->
        <!--                       {pot.name} -->
        <!--                     </div> -->
        <!--                     <div class="text-sm text-secondary-foreground"> -->
        <!--                       {createdAt} -->
        <!--                     </div> -->
        <!--                   </div> -->
        <!--                 </Button> -->
        <!--               </li> -->
        <!--             {/each} -->
        <!--           </ul> -->
        <!--         </ScrollArea> -->
        <!--       </Dialog.Description> -->
        <!--     </Dialog.Header> -->
        <!--   </Dialog.Content> -->
        <!-- </Dialog.Root> -->
      </div>
    {/if}
  {/await}
</div>
