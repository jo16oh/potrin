import { View } from "$lib/models/Workspace.svelte";
import { getContext, setContext } from "svelte";

type AllowedViewType = "cards" | "outline" | "search";

export class HoverViewState {
  private static KEY = Symbol();

  static init(type: AllowedViewType = "cards") {
    setContext(this.KEY, new HoverViewState(type));
  }

  view = $state<View<AllowedViewType>>()!;
  open = $state(false);

  constructor(type: AllowedViewType) {
    this.view = View.new(type);
  }

  static get current() {
    return getContext<HoverViewState | undefined>(this.KEY);
  }
}
