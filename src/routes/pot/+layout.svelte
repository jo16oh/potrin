<script lang="ts">
  import type { LayoutData } from "./$types";
  import { Workspace } from "$lib/models/Workspace.svelte";
  import type { Snippet } from "svelte";

  let { data, children }: { data: LayoutData; children: Snippet } = $props();
  Workspace.init(data.workspaceState);
</script>

{@render children()}
