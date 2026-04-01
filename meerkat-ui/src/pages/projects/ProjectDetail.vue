<script setup lang="ts">
import { useCurrentProject } from '@/composables/useCurrentProject'

const { currentProject, slug } = useCurrentProject()
</script>

<template>
  <div>
    <h1 class="text-xl font-semibold text-foreground mb-1">{{ currentProject?.name ?? slug }}</h1>
    <p class="text-sm text-muted-foreground">Project details coming soon.</p>
  </div>
</template>
