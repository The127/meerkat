<script setup lang="ts">
import { MkButton } from '@/components/meerkat'
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-[60vh] text-center">
    <p class="text-6xl font-bold text-muted-foreground/30 font-mono">404</p>
    <h1 class="text-xl font-semibold text-foreground mt-4">Page not found</h1>
    <p class="text-sm text-muted-foreground mt-2 mb-6">The page you're looking for doesn't exist.</p>
    <MkButton variant="outline" as="RouterLink" to="/">Back to dashboard</MkButton>
  </div>
</template>
