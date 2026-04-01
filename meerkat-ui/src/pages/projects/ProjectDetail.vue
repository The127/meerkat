<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()
const slug = route.params.slug as string
</script>

<template>
  <div>
    <h1 class="text-xl font-semibold text-foreground mb-1">{{ slug }}</h1>
    <p class="text-sm text-muted-foreground">Project details coming soon.</p>
  </div>
</template>
