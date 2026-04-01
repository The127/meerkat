<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { MkButton } from '@/components/meerkat'

const error = ref<Error | null>(null)

onErrorCaptured((err) => {
  error.value = err
  return false
})

function retry() {
  error.value = null
}
</script>

<template>
  <slot v-if="!error" />
  <div v-else class="flex flex-col items-center justify-center min-h-[40vh] text-center px-4">
    <p class="text-5xl font-bold text-muted-foreground/30 font-mono">!</p>
    <h2 class="text-lg font-semibold text-foreground mt-4">Something went wrong</h2>
    <p class="text-sm text-muted-foreground mt-1 max-w-md">{{ error.message }}</p>
    <MkButton variant="outline" class="mt-6" @click="retry">Try again</MkButton>
  </div>
</template>
