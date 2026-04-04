<script setup lang="ts">
import MkButton from './MkButton.vue'

defineProps<{
  currentPage: number
  totalPages: number
  hasPrev: boolean
  hasNext: boolean
}>()

defineEmits<{
  prev: []
  next: []
}>()
</script>

<template>
  <div v-if="totalPages > 1" class="flex items-center justify-between pt-2">
    <span class="text-xs text-muted-foreground">
      Page {{ currentPage }} of {{ totalPages }}
    </span>
    <div class="flex gap-2">
      <MkButton size="sm" variant="outline" :disabled="!hasPrev" @click="$emit('prev')">
        Previous
      </MkButton>
      <MkButton size="sm" variant="outline" :disabled="!hasNext" @click="$emit('next')">
        Next
      </MkButton>
    </div>
  </div>
</template>
