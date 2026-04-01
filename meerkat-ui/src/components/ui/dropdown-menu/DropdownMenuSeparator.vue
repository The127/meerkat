<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { DropdownMenuSeparator } from 'radix-vue'
import { cn } from '@/lib/utils'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DropdownMenuSeparator :class="cn('-mx-1 my-1 h-px bg-border', props.class)" />
</template>
