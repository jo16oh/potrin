<script lang="ts">
  import type { SVGAttributes } from "svelte/elements";
  let { ...restProps }: SVGAttributes<SVGElement> = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" {...restProps}>
  <line
    x1="50%"
    x2="50%"
    y1="0"
    y2="100%"
    stroke="currentColor"
    stroke-width="0.0625rem"
    stroke-linecap="round"
    stroke-dasharray="2 3 2 1.5"
  />
</svg>
