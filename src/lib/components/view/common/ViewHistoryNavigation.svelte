<script lang="ts">
  import Button from "$lib/components/common/Button.svelte";
  import { View } from "$lib/models/Workspace.svelte";
  import { ChevronLeft, ChevronRight } from "lucide-svelte";
  import { css, type Styles } from "styled-system/css";

  type Props = {
    view: View;
    buttonStyle: Styles;
    iconStyle: Styles;
  };

  let { view, buttonStyle, iconStyle }: Props = $props();
</script>

<Button
  class={css(buttonStyle)}
  disabled={!View.hasPrev(view.id)}
  onmousedown={(e: MouseEvent) => e.preventDefault()}
  onclick={(e: MouseEvent) => {
    e.preventDefault();
    View.back(view);
  }}
>
  <ChevronLeft class={css(iconStyle)} data-disabled={!View.hasPrev(view.id)} />
</Button>
<Button
  class={css(buttonStyle)}
  disabled={!View.hasNext(view.id)}
  onmousedown={(e: MouseEvent) => e.preventDefault()}
  onclick={(e: MouseEvent) => {
    e.preventDefault();
    View.forward(view);
  }}
>
  <ChevronRight class={css(iconStyle)} data-disabled={!View.hasNext(view.id)} />
</Button>
