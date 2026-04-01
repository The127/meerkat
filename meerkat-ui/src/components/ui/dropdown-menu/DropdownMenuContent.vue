<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { DropdownMenuContent, DropdownMenuPortal, type DropdownMenuContentProps, useForwardPropsEmits } from 'radix-vue'
import { cn } from '@/lib/utils'

interface Props extends DropdownMenuContentProps {
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  sideOffset: 4,
  align: 'end',
})

const emits = defineEmits<{
  closeAutoFocus: [event: Event]
}>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuContent
      v-bind="forwarded"
      :class="cn(
        'z-50 min-w-[10rem] overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md',
        'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
        props.class,
      )"
    >
      <slot />
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
