<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuth } from '@/composables/useAuth'
import { MkSpinner } from '@/components/meerkat'

const router = useRouter()
const { handleCallback } = useAuth()

onMounted(async () => {
  try {
    await handleCallback()
    router.replace('/')
  } catch {
    router.replace('/login')
  }
})
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background">
    <div class="flex flex-col items-center gap-3">
      <MkSpinner size="lg" />
      <p class="text-sm text-muted-foreground">Completing sign in...</p>
    </div>
  </div>
</template>
