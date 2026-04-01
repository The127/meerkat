<script setup lang="ts">
import { MkButton } from '@/components/meerkat'
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-screen text-center px-4">
    <p class="text-6xl font-bold text-muted-foreground/30 font-mono">500</p>
    <h1 class="text-xl font-semibold text-foreground mt-4">Something went wrong</h1>
    <p class="text-sm text-muted-foreground mt-2 mb-6">An unexpected error occurred. Please try again.</p>
    <MkButton variant="outline" as="a" href="/">Back to dashboard</MkButton>
  </div>
</template>
