<script lang="ts">
  import * as Sidebar from "./ui/sidebar/index";
  import * as DropdownMenu from "./ui/dropdown-menu/index";
  import { ChevronDownIcon, CookingPotIcon } from "lucide-svelte";
  import { App } from "$lib/models/App.svelte";

  const appState = App.state();
</script>

<Sidebar.Menu class="group/swither">
  <Sidebar.MenuItem>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuButton
            size="lg"
            class="group/button flex px-1 data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            {...props}
          >
            <div
              class="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-sidebar-primary-foreground"
            >
              <CookingPotIcon class="size-4" />
            </div>

            <h1 class="flex-grow truncate font-semibold">
              {appState.pot?.name}
            </h1>

            <div
              class="hidden h-full w-6 items-center justify-center rounded-lg group-hover/button:flex"
            >
              <ChevronDownIcon class="size-4" />
            </div>
          </Sidebar.MenuButton>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        class="w-[--bits-dropdown-menu-anchor-width]"
        align="start"
      >
        <DropdownMenu.Item onSelect={() => (appState.pot = null)}>
          Select Pot
        </DropdownMenu.Item>
        <DropdownMenu.Item onSelect={() => (appState.pot = null)}>
          Rename Pot
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
