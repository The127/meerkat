<script setup lang="ts">
import { type Component } from 'vue'

defineProps<{
  icon?: Component
  title: string
  description?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-16 text-center">
    <component
      :is="icon"
      v-if="icon"
      class="h-10 w-10 text-muted-foreground/50 mb-4"
    />
    <h3 class="text-sm font-medium text-foreground">{{ title }}</h3>
    <p v-if="description" class="mt-1 text-sm text-muted-foreground max-w-sm">
      {{ description }}
    </p>
    <div v-if="$slots.default" class="mt-4">
      <slot />
    </div>
  </div>
</template>
