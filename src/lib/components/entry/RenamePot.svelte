<script lang="ts">
  import { css, type Styles } from "styled-system/css";
  import Dialog from "$lib/components/common/Dialog.svelte";
  import { commands, type Pot } from "generated/tauri-commands";
  import { unwrap } from "$lib/utils";
  import { App } from "$lib/models/App.svelte";
  import type { Snippet } from "svelte";
  import CloseButton from "./CloseButton.svelte";
  import DialogClose from "$lib/components/common/DialogClose.svelte";
  import PopoverClose from "$lib/components/common/PopoverClose.svelte";

  let {
    pot,
    button,
    buttonStyle,
    onSubmit,
    open = $bindable(false),
  }: {
    pot: Pot;
    button: Snippet;
    buttonStyle: Styles;
    onSubmit?: () => void;
    open?: boolean;
  } = $props();

  const appState = App.state();

  const minNameLength = 1;
  const maxNameLength = 50;

  let name = $state("");
  let canSubmit = $state(true);

  let input: HTMLInputElement;

  $effect(() => {
    if (open) {
      name = "";
    }
  });

  $effect(() => {
    canSubmit = !(minNameLength <= name.length && name.length <= maxNameLength);
  });

  async function updatePot() {
    open = false;
    const newPot: Pot = { ...pot, name: name };

    await commands.updatePot(newPot).then(unwrap);

    appState.pots[pot.id] = pot.name;
    onSubmit?.();
  }
</script>

<Dialog
  bind:open
  triggerStyle={buttonStyle}
  contentProps={{
    onOpenAutoFocus: (e) => {
      e.preventDefault();
      input.focus();
    },
  }}
>
  {#snippet trigger()}
    {@render button()}
  {/snippet}
  {#snippet content()}
    <div class={titleStyle}>
      <div>Rename your pot</div>
      <DialogClose>
        <CloseButton />
      </DialogClose>
    </div>
    <div class={contentStyle}>
      <input
        bind:this={input}
        bind:value={name}
        placeholder="Name your pot..."
        class={inputStyle}
      />
      <div
        class={css({
          color: "view.text-muted",
        })}
      >
        <span class={canSubmit ? css({ color: "[red]" }) : ""}
          >{name.length}</span
        > / 50
      </div>
      <div
        class={css({
          w: "full",
          h: "2",
        })}
      ></div>
      <PopoverClose
        class={closeButtonStyle}
        disabled={canSubmit}
        onclick={updatePot}
      >
        Rename
      </PopoverClose>
    </div>
  {/snippet}
</Dialog>

<script module>
  const titleStyle = css({
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    p: "1",
    userSelect: "none",
  });

  const contentStyle = css({
    w: "full",
    h: "fit",
    p: "2",
    display: "flex",
    flexDir: "column",
    gap: "2",
    justifyContent: "start",
    alignItems: "end",
    userSelect: "none",
  });

  const inputStyle = css({
    w: "full",
    h: "9",
    color: "view.text",
    borderBottomWidth: "thin",
    borderColor: "view.text",
    ring: "none",
  });

  const closeButtonStyle = css({
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    gap: "2",
    paddingX: "4",
    paddingY: "2",
    bg: "button.bg",
    shadow: "sm",
    rounded: "lg",
    width: "fit",
    height: "fit",
    _hover: {
      bg: "darken",
    },
    _disabled: {
      color: "button.text-muted",
      bg: "button.bg/20",
      _hover: {
        bg: "button.bg/20",
      },
    },
    transition: "colors",
  });
</script>
