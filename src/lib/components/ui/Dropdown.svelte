<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { fly } from "svelte/transition";
  import { cn } from "$lib/shared/cn";
  import { ChevronDown, Check } from "lucide-svelte";

  type DropdownItem = {
    label: string;
    value: string;
    icon?: any;
    destructive?: boolean;
    disabled?: boolean;
  };

  interface Props {
    id?: string;
    label?: string;
    value?: string;
    items: DropdownItem[];
    placeholder?: string;
    onSelect?: (value: string) => void;
    class?: string;
    triggerClass?: string;
    contentClass?: string;
    align?: "start" | "center" | "end";
    sideOffset?: number;
  }

  let {
    id,
    label,
    value = $bindable(),
    items,
    placeholder = "Select an option",
    onSelect,
    class: className,
    triggerClass,
    contentClass,
    align = "start",
    sideOffset = 8,
  }: Props = $props();

  let selectedItem = $derived(items.find((i) => i.value === value));
</script>

<div {id} class={cn("dropdown-container", className)}>
  {#if label}
    <span class="mb-2 block text-sm font-medium text-muted-foreground">
      {label}
    </span>
  {/if}

  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={cn(
        "flex h-11 w-full items-center justify-between rounded-xl border border-white/10 bg-white/5 px-4 text-sm font-medium transition-all hover:bg-white/10 hover:border-white/20 active:scale-[0.98] outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
        triggerClass,
      )}
    >
      <span class={cn(!selectedItem && "text-muted-foreground")}>
        {selectedItem ? selectedItem.label : placeholder}
      </span>
      <ChevronDown
        class="ml-2 size-4 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180"
      />
    </DropdownMenu.Trigger>

    <DropdownMenu.Portal>
      <DropdownMenu.Content
        {align}
        {sideOffset}
        class={cn(
          "z-100 min-w-[200px] overflow-hidden rounded-2xl border border-white/10 bg-zinc-950/90 p-1.5 shadow-2xl backdrop-blur-xl outline-none",
          contentClass,
        )}
        forceMount
      >
        {#snippet child({ wrapperProps, props, open })}
          {#if open}
            <div {...wrapperProps}>
              <div {...props} transition:fly={{ y: 8, duration: 200 }}>
                {#each items as item (item.value)}
                  <DropdownMenu.Item
                    onSelect={() => {
                      value = item.value;
                      onSelect?.(item.value);
                    }}
                    disabled={item.disabled}
                    class={cn(
                      "flex cursor-default select-none items-center rounded-xl px-3 py-2.5 text-sm font-medium outline-none transition-colors",
                      "data-highlighted:bg-white/10 data-highlighted:text-foreground",
                      "data-disabled:pointer-events-none data-disabled:opacity-50",
                      item.destructive &&
                        "text-red-400 data-highlighted:bg-red-500/20 data-highlighted:text-red-400",
                    )}
                  >
                    {#if item.icon}
                      <item.icon class="mr-2.5 size-4 opacity-70" />
                    {/if}
                    <span>{item.label}</span>

                    {#if value === item.value}
                      <Check class="ml-auto size-4 text-primary" />
                    {/if}
                  </DropdownMenu.Item>
                {/each}
              </div>
            </div>
          {/if}
        {/snippet}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</div>
