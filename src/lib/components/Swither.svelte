<script lang="ts">
  import * as Sidebar from "./ui/sidebar/index";
  import * as DropdownMenu from "./ui/dropdown-menu/index";
  import { ChevronDownIcon, CookingPotIcon } from "lucide-svelte";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import { commands } from "../../generated/tauri-commands";

  const workspace = Workspace.state();
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

            <div class="flex-grow truncate font-semibold">
              {workspace.pot.name}
            </div>

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
        <DropdownMenu.Item
          onSelect={() => {
            commands.openPotSelector();
          }}
        >
          Select Pot
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
