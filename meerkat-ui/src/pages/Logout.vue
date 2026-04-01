<script setup lang="ts">
import { onMounted } from 'vue'
import { useAuth } from '@/composables/useAuth'
import { MkSpinner } from '@/components/meerkat'

const { logout } = useAuth()

onMounted(async () => {
  await logout()
})
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-3">
      <MkSpinner size="lg" />
      <p class="text-sm text-muted-foreground">Signing out...</p>
    </div>
  </div>
</template>
