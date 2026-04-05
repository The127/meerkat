<script setup lang="ts">
import { ChevronLeft } from 'lucide-vue-next'

defineProps<{
  to: string | Record<string, unknown>
}>()
</script>

<template>
  <RouterLink
    :to="to"
    class="mb-4 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
  >
    <ChevronLeft class="w-4 h-4" />
    <slot />
  </RouterLink>
</template>
