<script lang="ts">
  import { css } from "styled-system/css";
  import Dialog from "$lib/components/common/Dialog.svelte";
  import Button, { buttonStyle } from "$lib/components/common/Button.svelte";
  import { commands } from "../../../generated/tauri-commands";
  import { uuidv7, unwrap } from "$lib/utils";
  import { App } from "$lib/models/App.svelte";
  import CloseButton from "./CloseButton.svelte";
  // import { getCurrent } from "@tauri-apps/api/window";

  const [_, updateAppState] = App.state();

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

    unwrap(await commands.createPot(pot));

    updateAppState((state) => {
      state.pots[pot.id] = pot.name;
      return state;
    });

    unwrap(await commands.openPot(pot.id, pot.name));

    // getCurrent().close();
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
    onCloseAutoFocus: (e) => {
      e.preventDefault();
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
      <Button bind:disabled={canSubmit} onclick={createPot}>Craete</Button>
    </div>
  {/snippet}
</Dialog>

<script module>
  const createNewPotButtonStyle = css.raw({
    ...buttonStyle,
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
</script>
