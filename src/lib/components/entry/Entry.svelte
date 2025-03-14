<script lang="ts">
  import { commands } from "generated/tauri-commands";
  import Button from "$lib/components/common/Button.svelte";
  import UserIcon from "$lib/components/icon/UserIcon.svelte";
  import { css } from "styled-system/css";
  import { ChevronDown, Globe } from "lucide-svelte";
  import CreateNewPotButton from "$lib/components/entry/CreateNewPotButton.svelte";
  import SelectPotButton from "$lib/components/entry/SelectPotButton.svelte";

  const version = commands.appVersion();
</script>

<div
  class={css({
    bg: "view.bg",
    display: "flex",
    flexDir: "column",
    alignItems: "center",
    w: "screen",
    h: "screen",
    userSelect: "none",
    overscrollBehavior: "none",
    overflow: "hidden",
  })}
>
  <div
    class={css({
      w: "full",
      h: "14",
    })}
  ></div>

  <img
    src="Pot.svg"
    alt="Potrin logo"
    class={css({
      h: "44",
      w: "44",
      p: "4",
      filter: "auto",
      dropShadow: "md",
    })}
  />

  <img
    src="PotrinLogo.svg"
    alt="Potrin"
    class={css({
      h: "8",
    })}
  />

  <div
    class={css({
      fontSize: "sm",
      p: "2",
    })}
  >
    {#await version then version}
      Version {version}
    {/await}
  </div>

  <div
    class={css({
      w: "full",
      h: "10",
    })}
  ></div>
  <div
    class={css({
      w: "56",
      py: "4",
      px: "4",
      borderBottomWidth: "1",
    })}
  >
    <Button
      class={css({
        ...buttonStyle,
        w: "full",
        h: "9",
      })}
      disabled={true}
    >
      <UserIcon />
      <div class={css({ flex: "1", color: "button.text-muted" })}>
        Local user
      </div>
      <ChevronDown
        class={css({
          color: "button.text-muted",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          w: "4",
          h: "4",
        })}
      />
    </Button>
  </div>
  <div
    class={css({
      display: "flex",
      flexDir: "column",
      justifyContent: "start",
      alignItems: "center",
      gap: "4",
      w: "56",
      px: "4",
      py: "4",
      h: "fit",
    })}
  >
    <CreateNewPotButton />
    <SelectPotButton />

    <Button
      class={css({
        ...buttonStyle,
        w: "full",
        h: "9",
      })}
      disabled={true}
    >
      <div class={css({ color: "button.text-muted" })}>Setting</div>
    </Button>
  </div>
  <Button
    class={css({
      ...buttonStyle,
      position: "fixed",
      top: "4",
      right: "4",
      w: "fit",
      h: "9",
    })}
    disabled={true}
  >
    <Globe
      class={css({
        color: "button.text-muted",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        w: "4",
        h: "4",
      })}
    ></Globe>
    <div class={css({ flex: "1", color: "button.text-muted" })}>English</div>
    <ChevronDown
      class={css({
        color: "button.text-muted",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        w: "4",
        h: "4",
      })}
    />
  </Button>
</div>

<script module>
  const buttonStyle = css.raw({
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
