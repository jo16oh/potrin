<script lang="ts">
  import { Asterisk, PencilLine } from "lucide-svelte";
  import Button from "./ui/button/button.svelte";
  import * as Dialog from "./ui/dialog/index";
</script>

<Dialog.Root>
  <Dialog.Trigger>
    <Button class="fixed bottom-5 right-8 aspect-square size-12 rounded-full ">
      <PencilLine />
    </Button>
  </Dialog.Trigger>
  <Dialog.Content
    class="max-w-screen flex h-screen w-screen flex-col items-start justify-start bg-background shadow-none"
  >
    <div class="flex h-9 flex-row items-center justify-start text-3xl">
      <Asterisk />
    </div>
    <div class="min-h-20 w-full rounded-md bg-card text-xl shadow-sm">
      content
    </div>
  </Dialog.Content>
</Dialog.Root>
