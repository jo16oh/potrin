<script lang="ts">
  import type { SVGAttributes } from "svelte/elements";
  let { ...restProps }: SVGAttributes<SVGElement> = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" {...restProps}>
  <line
    y1="0"
    y2="100%"
    stroke="currentColor"
    stroke-width="0.125rem"
    stroke-linecap="round"
  />
</svg>
