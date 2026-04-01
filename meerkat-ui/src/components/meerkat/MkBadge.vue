<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { Badge, type BadgeVariants } from '@/components/ui/badge'

defineProps<{
  variant?: BadgeVariants['variant']
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Badge :variant="variant" :class="$props.class">
    <slot />
  </Badge>
</template>
