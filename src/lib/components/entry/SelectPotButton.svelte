<script lang="ts">
  import { css } from "styled-system/css";
  import { commands, type Pot } from "../../../generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import Button, { buttonStyle } from "$lib/components/common/Button.svelte";
  import Dialog from "$lib/components/common/Dialog.svelte";
  import { App } from "$lib/models/App.svelte";
  import ScrollArea from "$lib/components/common/ScrollArea.svelte";
  import CloseButton from "./CloseButton.svelte";
  import { MoreHorizontal, PencilLine, Trash } from "lucide-svelte";
  import Popover from "$lib/components/common/Popover.svelte";
  import DialogClose from "$lib/components/common/DialogClose.svelte";
  import RenamePot from "./RenamePot.svelte";
  import { getCurrent } from "@tauri-apps/api/webviewWindow";

  const [getAppState, updateAppState] = App.state();
  const appState = $derived.by(getAppState);

  let open = $state(false);
  let potsPromise = $state(commands.fetchPots());

  async function reloadPots() {
    commands.fetchPots().then((pots) => {
      potsPromise = Promise.resolve(pots);
      if (unwrap(pots).length === 0) {
        open = false;
      }
    });
  }

  $effect(() => {
    if (open) {
      reloadPots();
    }
  });

  $effect(() => {
    if (Object.keys(appState.pots).length > 0) {
      reloadPots();
    }
  });

  async function openPot(id: string, name: string) {
    updateAppState((state) => {
      state.pots[id] = name;
      return state;
    });

    await commands.openPot(id).then(unwrap);
    getCurrent().close();
  }
</script>

{#await potsPromise}
  <Button disabled={true} style={selectPotButtonStyle}>Open pot</Button>
{:then potsResult}
  {@const pots = unwrap(potsResult)}
  <Dialog
    bind:open
    triggerProps={{ disabled: pots.length === 0 }}
    triggerStyle={{ ...buttonStyle, ...selectPotButtonStyle }}
  >
    {#snippet trigger()}
      <div class={css({ _disabled: { color: "button.text" } })}>Open pot</div>
    {/snippet}
    {#snippet content()}
      <div class={titleStyle}>
        <div>Select pot</div>
        <DialogClose>
          <CloseButton />
        </DialogClose>
      </div>
      <div>
        <ScrollArea orientation="vertical" type="scroll">
          <div class={potsContainerStyle}>
            {#each pots as pot}
              <button
                class={potStyle}
                onclick={() => openPot(pot.id, pot.name)}
              >
                <div class={outerPotTitleContainer}>
                  <div class={innerPotTitleContainer}>
                    <div class={potTitle}>
                      {pot.name}
                    </div>
                    {@render moreOperationButton(pot)}
                  </div>
                  <div class={potDescriptionContainer}>
                    <p class={potDescription}>
                      owner: {pot.owner ? pot.owner : "local_user"}
                    </p>
                    <p class={potDescription}>
                      created_at: {new Date(pot.createdAt).toLocaleDateString()}
                    </p>
                  </div>
                </div>
              </button>
            {/each}
          </div>
        </ScrollArea>
      </div>
    {/snippet}
  </Dialog>
{/await}

{#snippet moreOperationButton(pot: Pot)}
  <Popover
    triggerStyle={potOperationTriggerStyle}
    contentStyle={potOperationContainerStyle}
    triggerProps={{
      onclick: (e) => {
        e.stopPropagation();
      },
    }}
    contentProps={{
      align: "start",
      onclick: (e) => {
        e.stopPropagation();
      },
    }}
  >
    {#snippet trigger()}
      <MoreHorizontal class={iconStyle} />
    {/snippet}
    {#snippet content()}
      {@render renamePotDialog(pot)}
      {@render deletePotDialog(pot)}
    {/snippet}
  </Popover>
{/snippet}

{#snippet renamePotDialog(pot: Pot)}
  <RenamePot {pot} buttonStyle={renamePotButtonStyle} onSubmit={reloadPots}>
    {#snippet button()}
      <PencilLine class={iconStyle} />
      Rename pot
    {/snippet}
  </RenamePot>
{/snippet}

{#snippet deletePotDialog(pot: Pot)}
  <Dialog
    triggerStyle={deletePotButtonStyle}
    contentStyle={deletePotDialogContainerStyle}
  >
    {#snippet trigger()}
      <Trash class={iconStyle} />
      Delete pot
    {/snippet}
    {#snippet content()}
      <div class={deletePotDialogMessageStyle}>
        This operation is irreversible. Are you sure?
      </div>
      <div class={alignRightContainer}>
        <DialogClose
          class={deletePotDialogCloseButton}
          onclick={async () => {
            await commands.deletePot(pot.id).then(unwrap);
            updateAppState((state) => {
              delete state.pots[pot.id];
              return state;
            });
            await reloadPots();
          }}
        >
          Delete
        </DialogClose>
      </div>
    {/snippet}
  </Dialog>
{/snippet}

<script module>
  const selectPotButtonStyle = css.raw({
    w: "full",
    h: "9",
    color: "button.text",
  });

  const iconStyle = css({
    color: "view.text-muted",
    w: "4",
    h: "4",
  });

  const titleStyle = css({
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    p: "1",
    userSelect: "none",
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
    borderBottomWidth: "thin",
    borderBottomColor: "view.text-muted",
  });

  const outerPotTitleContainer = css({
    px: "1",
    py: "1",
    mb: "1",
    rounded: "md",
    transition: "background",
    _hover: {
      bg: "selected",
    },
  });

  const innerPotTitleContainer = css({
    display: "flex",
    flexDir: "row",
    justifyContent: "space-between",
    gap: "8",
  });

  const potTitle = css({
    textAlign: "start",
    flex: "1",
    w: "full",
    fontSize: "lg",
    overflowWrap: "anywhere",
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

  const potOperationTriggerStyle = css.raw({
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
    w: "6",
    h: "6",
    rounded: "circle",
    transition: "background",
    _hover: {
      bg: "selected",
    },
  });

  const potOperationContainerStyle = css.raw({
    bg: "view.bg",
    p: "1",
  });

  const renamePotButtonStyle = css.raw({
    ...buttonStyle,
    justifyContent: "start",
    fontSize: "sm",
    w: "full",
    p: "1",
    rounded: "[0.25rem]",
    bg: "transparent",
    shadow: "[]",
  });

  const deletePotButtonStyle = css.raw({
    ...buttonStyle,
    color: "[red]",
    justifyContent: "start",
    fontSize: "sm",
    w: "full",
    p: "1",
    rounded: "[0.25rem]",
    bg: "transparent",
    shadow: "[]",
  });

  const deletePotDialogContainerStyle = css.raw({
    w: "fit",
    minH: "[]",
  });

  const deletePotDialogMessageStyle = css({
    fontSize: "xl",
    p: "1",
  });

  const deletePotDialogCloseButton = css({
    ...buttonStyle,
    bg: "[red]",
    color: "[white]",
    p: "1",
    rounded: "md",
  });

  const alignRightContainer = css({
    w: "full",
    display: "flex",
    flexDir: "row",
    justifyContent: "end",
  });
</script>
