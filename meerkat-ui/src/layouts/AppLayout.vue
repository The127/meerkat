<script setup lang="ts">
import MkSidebar from '@/components/meerkat/MkSidebar.vue'
import MkTopbar from '@/components/meerkat/MkTopbar.vue'
import MkErrorBoundary from '@/components/meerkat/MkErrorBoundary.vue'
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background">
    <MkSidebar />
    <div class="flex flex-col flex-1 overflow-hidden">
      <MkTopbar />
      <main class="flex-1 overflow-auto p-6">
        <MkErrorBoundary>
          <RouterView />
        </MkErrorBoundary>
      </main>
    </div>
  </div>
</template>
