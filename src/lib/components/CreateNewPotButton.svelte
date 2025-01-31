<script lang="ts">
  import { css } from "styled-system/css";
  import { X } from "lucide-svelte";
  import Dialog from "./Dialog.svelte";
  import Button from "./Button.svelte";
  import { commands } from "../../generated/tauri-commands";
  import { uuidv7, unwrap } from "$lib/utils";
  import { App } from "$lib/models/App.svelte";
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
    });

    unwrap(await commands.openPot(pot.id, pot.name));

    // getCurrent().close();
  }
</script>

<Dialog bind:open>
  {#snippet trigger()}
    <Button
      style={css.raw({
        w: "full",
        h: "9",
      })}
      onclick={() => {
        setTimeout(() => {
          input.focus();
        }, 150);
      }}
    >
      <div class={css({ color: "button.text" })}>Create new Pot</div>
    </Button>
  {/snippet}
  {#snippet title()}
    <div class={titleContainerStyle}>
      <div>Create new Pot</div>
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
    <div class={descriptionContainerStyle}>
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

  const descriptionContainerStyle = css({
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
