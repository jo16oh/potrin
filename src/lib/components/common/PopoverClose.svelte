<script lang="ts">
  import { Popover, type PopoverCloseProps } from "bits-ui";

  let { ...restProps }: PopoverCloseProps = $props();
</script>

<Popover.Close {...restProps}></Popover.Close>
