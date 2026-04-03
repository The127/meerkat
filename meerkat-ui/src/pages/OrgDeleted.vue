<script setup lang="ts">
import { MkButton } from '@/components/meerkat'
import { Trash2 } from 'lucide-vue-next'
</script>

<template>
  <div class="flex min-h-screen items-center justify-center px-4">
    <div class="text-center max-w-sm">
      <div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
        <Trash2 class="h-6 w-6 text-destructive" />
      </div>
      <h1 class="text-xl font-semibold text-foreground mb-2">Organization deleted</h1>
      <p class="text-sm text-muted-foreground mb-6">
        This organization has been permanently deleted along with all its projects and data.
      </p>
      <RouterLink to="/auth/login">
        <MkButton>Back to login</MkButton>
      </RouterLink>
    </div>
  </div>
</template>
