<script lang="ts">
  import { Button } from "bits-ui";

  let { children, ...others }: Button.RootProps = $props();
</script>

<Button.Root {...others}>
  {@render children?.()}
</Button.Root>
