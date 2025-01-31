<script lang="ts">
  import { css } from "styled-system/css";
  import Button from "$lib/components/Button.svelte";
  import Dialog from "$lib/components/Dialog.svelte";
  import { X } from "lucide-svelte";
  import { App } from "$lib/models/App.svelte";
  import { commands } from "../../generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import ScrollArea from "./ScrollArea.svelte";
  // import { getCurrent } from "@tauri-apps/api/webviewWindow";

  const [_, updateAppState] = App.state();

  let open = $state(false);
  let pots = $state(commands.fetchPots());

  $effect(() => {
    if (open) {
      pots = commands.fetchPots();
    }
  });

  async function openPot(id: string, name: string) {
    open = false;

    updateAppState((state) => {
      state.pots[id] = name;
    });

    unwrap(await commands.openPot(id, name));
    // getCurrent().close();
  }
</script>

<Dialog bind:open>
  {#snippet trigger()}
    <Button
      style={css.raw({
        w: "full",
        h: "9",
        color: "button.text",
      })}
    >
      <div class={css({ _disabled: { color: "button.text" } })}>Open pot</div>
    </Button>
  {/snippet}
  {#snippet title()}
    <div class={titleContainerStyle}>
      <div>Select pot</div>
      <button onclick={() => (open = false)} class={closeButtonStyle}>
        <X
          class={css({
            color: "view.text-muted",
            w: "4",
            h: "4",
          })}
        />
      </button>
    </div>
  {/snippet}
  {#snippet description()}
    <ScrollArea orientation="vertical" type="scroll">
      <div class={potsContainerStyle}>
        {#await pots then potsResult}
          {#each unwrap(potsResult) as pot}
            <button class={potStyle} onclick={() => openPot(pot.id, pot.name)}>
              <div class={potTitleContainer}>
                <div class={potTitle}>
                  {pot.name}
                </div>
              </div>
              <div class={potDescriptionContainer}>
                <p class={potDescription}>
                  owner: {pot.owner ? pot.owner : "local_user"}
                </p>
                <p class={potDescription}>
                  created_at: {new Date(pot.createdAt).toLocaleDateString()}
                </p>
              </div>
            </button>
          {/each}
        {/await}
      </div>
    </ScrollArea>
  {/snippet}
</Dialog>

<script module>
  const titleContainerStyle = css({
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    p: "1",
    userSelect: "none",
  });

  const closeButtonStyle = css({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    w: "6",
    h: "6",
    rounded: "lg",
    transition: "background",
    _hover: {
      bg: "selected",
    },
  });

  const potsContainerStyle = css({
    display: "flex",
    flexDir: "column",
    gap: "2",
    w: "full",
    h: "fit",
    maxH: "[60vh]",
    p: "2",
    userSelect: "none",
  });

  const potStyle = css({
    w: "full",
    h: "fit",
    px: "1",
    py: "2",
    borderBottomWidth: "thin",
    borderBottomColor: "view.text-muted",
    transition: "background",
    _hover: {
      bg: "selected",
    },
  });

  const potTitleContainer = css({
    display: "flex",
    flexDir: "row",
    justifyContent: "space-between",
    gap: "8",
  });

  const potTitle = css({
    fontSize: "lg",
    color: "view.text",
  });

  const potDescriptionContainer = css({
    px: "2",
  });

  const potDescription = css({
    color: "view.text-muted",
    fontSize: "xs",
    w: "fit",
  });
</script>
