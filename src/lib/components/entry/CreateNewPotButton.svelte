<script lang="ts">
  import { css } from "styled-system/css";
  import Dialog from "$lib/components/common/Dialog.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import { commands } from "generated/tauri-commands";
  import { uuidv7, unwrap } from "$lib/utils";
  import { App } from "$lib/models/App.svelte";
  import CloseButton from "./CloseButton.svelte";
  import { getCurrent } from "@tauri-apps/api/window";

  const appState = App.state();

  const minNameLength = 1;
  const maxNameLength = 50;

  let open = $state(false);
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

  async function createPot() {
    open = false;

    const pot = {
      id: uuidv7(),
      owner: null,
      name: name,
      createdAt: new Date().getMilliseconds(),
    };

    await commands.createPot(pot).then(unwrap);

    appState.pots[pot.id] = pot.name;

    await commands.openPot(pot.id).then(unwrap);

    getCurrent().close();
  }
</script>

<Dialog
  bind:open
  triggerStyle={createNewPotButtonStyle}
  contentProps={{
    onOpenAutoFocus: (e) => {
      e.preventDefault();
      input.focus();
    },
  }}
>
  {#snippet trigger()}
    Create new Pot
  {/snippet}
  {#snippet content()}
    <div class={titleStyle}>
      <div>Create new Pot</div>
      <CloseButton onclick={() => (open = false)} />
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
      <Button class={submitButtonStyle} disabled={canSubmit} onclick={createPot}
        >Craete</Button
      >
    </div>
  {/snippet}
</Dialog>

<script module>
  const createNewPotButtonStyle = css.raw({
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    px: "4",
    py: "2",
    bg: "button.bg",
    shadow: "sm",
    rounded: "lg",
    _hover: {
      bg: "button.bg-selected",
    },
    transition: "colors",
    w: "full",
    h: "9",
  });
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

  const submitButtonStyle = css({
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
      bg: "button.bg-selected",
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
