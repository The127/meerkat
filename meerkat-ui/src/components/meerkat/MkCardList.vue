<script setup lang="ts" generic="T">
import { type Component } from 'vue'
import MkSpinner from './MkSpinner.vue'
import MkEmptyState from './MkEmptyState.vue'

defineProps<{
  items: T[]
  loading?: boolean
  emptyTitle?: string
  emptyDescription?: string
  emptyIcon?: Component
}>()

defineSlots<{
  item(props: { item: T }): unknown
  empty(): unknown
}>()
</script>

<template>
  <div v-if="loading" class="flex items-center justify-center py-16">
    <MkSpinner size="lg" />
  </div>

  <MkEmptyState
    v-else-if="items.length === 0"
    :icon="emptyIcon"
    :title="emptyTitle ?? 'No items'"
    :description="emptyDescription"
  >
    <slot name="empty" />
  </MkEmptyState>

  <div v-else class="flex flex-col gap-2">
    <div
      v-for="(item, index) in items"
      :key="index"
      class="rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent/50"
    >
      <slot name="item" :item="item" />
    </div>
  </div>
</template>
