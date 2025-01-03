<script lang="ts">
  import * as Sidebar from "./ui/sidebar/index";
  import {
    AsteriskIcon,
    CalendarArrowDown,
    PanelLeft,
    Search,
    Settings,
    X,
  } from "lucide-svelte";
  import Swither from "./Swither.svelte";
  import Button from "./ui/button/button.svelte";
  import type { TabState } from "../../generated/tauri-commands";
  import { uuidv7 } from "$lib/utils";
  import { cn } from "$lib/utils";
  import { Workspace } from "$lib/models/Workspace.svelte";

  const workspace = Workspace.state();
  const sidebar = $derived(workspace.sidebar);
</script>

<Sidebar.Header
  class="z-50 rounded-md border-b border-sidebar-border bg-sidebar px-1"
>
  <div class="flex flex-row items-center justify-between gap-1">
    <Swither />
    <Button
      variant="ghost"
      onclick={() => {
        sidebar.toggleFloat();
      }}
      class="flex h-full items-center justify-center rounded-lg focus-visible:ring-sidebar-ring"
    >
      <PanelLeft class="size-5 text-sidebar-foreground" />
    </Button>
  </div>
  <Sidebar.Menu>
    <Sidebar.MenuItem>
      <Sidebar.MenuButton
        class={cn(
          "flex gap-0 px-1",
          "timeline" in workspace.focus ? "bg-zinc-100" : "",
        )}
        onclick={() => {
          workspace.focusTo("timeline");
        }}
      >
        <div class="flex size-8 items-center justify-center rounded-lg">
          <CalendarArrowDown class="size-4" />
        </div>
        <div class="text-md select-none px-1 font-semibold">Timeline</div>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
    <Sidebar.MenuItem>
      <Sidebar.MenuButton
        class={cn(
          "flex gap-0 px-1",
          "search" in workspace.focus ? "bg-zinc-100" : "",
        )}
        onclick={() => {
          workspace.focusTo("search");
        }}
      >
        <div
          class="flex aspect-square size-8 items-center justify-center rounded-lg"
        >
          <Search class="size-4" />
        </div>
        <div class="text-md select-none px-1 font-semibold">Search</div>
      </Sidebar.MenuButton>
    </Sidebar.MenuItem>
  </Sidebar.Menu>
</Sidebar.Header>

<Sidebar.Content>
  <Sidebar.Group class="h-full">
    <Sidebar.GroupLabel>Tabs</Sidebar.GroupLabel>
    <Sidebar.GroupContent class="h-full">
      <Sidebar.Menu class="h-full">
        {@const tabs: TabState[] = [
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt",
                viewType: "outline",
                flexGrow: 10000
              },
              {
                id: uuidv7(),
                title: "Earl Sweatshirt",
                viewType: "outline",
                flexGrow: 10000
              },
              {
                id: uuidv7(),
                title: "Earl Sweatshirt",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        { 
            views: [
              {
                id: uuidv7(),
                title: "Earl Sweatshirt Earl Sweatshirt Earl",
                viewType: "outline",
                flexGrow: 10000
              },
            ],
            focusedViewIdx: 0
          },
        ]}
        {#each tabs as tab, tabIdx}
          <Sidebar.MenuItem>
            <Sidebar.MenuButton
              class={cn(
                "group/tabs mb-1 flex h-full items-center justify-start gap-1 p-1",
                "tabs" in workspace.focus &&
                  tabIdx === workspace.focus.tabs.index
                  ? "bg-zinc-100"
                  : "divide-x divide-sidebar-border",
              )}
              onclick={() => {
                workspace.focusTo(tabIdx);
              }}
            >
              {#each tab.views as view, viewIdx}
                <div class="flex-1 truncate">
                  <div
                    class={cn(
                      "flex items-center justify-start gap-1 truncate rounded-lg p-1",
                      tab.views.length !== 1 &&
                        "tabs" in workspace.focus &&
                        tabIdx === workspace.focus.tabs.index
                        ? "bg-zinc-200"
                        : "",
                    )}
                  >
                    <div
                      class="flex aspect-square items-center justify-center rounded-lg"
                    >
                      <AsteriskIcon class="size-4" />
                    </div>
                    <div class="text-md grow select-none truncate">
                      {view.title}
                    </div>
                    {#if ("tabs" in workspace.focus && tabIdx === workspace.focus.tabs.index) || viewIdx === tab.views.length - 1}
                      <div
                        class="hidden aspect-square h-full items-center justify-center rounded-lg text-sidebar-foreground/70 transition-colors duration-100 hover:text-sidebar-foreground group-hover/tabs:flex"
                      >
                        <X class="size-4" />
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        {/each}
      </Sidebar.Menu>
    </Sidebar.GroupContent>
  </Sidebar.Group>
</Sidebar.Content>

<Sidebar.Footer class="mx-1 border-t border-t-sidebar-border p-0">
  <Sidebar.Menu>
    <Sidebar.MenuItem class="px-1">
      <div
        class="flex aspect-square size-8 h-9 items-center justify-center rounded-lg"
      >
        <Sidebar.MenuButton>
          <Settings class="size-4" />
        </Sidebar.MenuButton>
      </div>
    </Sidebar.MenuItem>
  </Sidebar.Menu>
</Sidebar.Footer>
