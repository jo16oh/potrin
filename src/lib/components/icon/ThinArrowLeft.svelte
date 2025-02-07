<script lang="ts">
  import type { SVGAttributes } from "svelte/elements";
  let { ...restProps }: SVGAttributes<SVGElement> = $props();
</script>

<svg
  width="17"
  height="17"
  viewBox="0 0 17 8"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  {...restProps}
>
  <path
    d="M16.5 3.5C16.7761 3.5 17 3.72386 17 4C17 4.27614 16.7761 4.5 16.5 4.5V3.5ZM0.146446 4.35355C-0.0488148 4.15829 -0.0488148 3.84171 0.146446 3.64645L3.32843 0.464466C3.52369 0.269204 3.84027 0.269204 4.03553 0.464466C4.2308 0.659728 4.2308 0.976311 4.03553 1.17157L1.20711 4L4.03553 6.82843C4.2308 7.02369 4.2308 7.34027 4.03553 7.53553C3.84027 7.7308 3.52369 7.7308 3.32843 7.53553L0.146446 4.35355ZM16.5 4.5H0.5V3.5H16.5V4.5Z"
    fill="black"
  />
</svg>
