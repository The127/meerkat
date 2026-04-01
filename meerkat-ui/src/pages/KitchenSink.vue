<script setup lang="ts">
import { ref } from 'vue'
import { MkButton, MkInput, MkCard, MkBadge, MkAlert, MkSpinner } from '@/components/meerkat'
import { useToast } from '@/composables/useToast'

const inputValue = ref('')
const toast = useToast()
</script>

<template>
  <div class="min-h-screen bg-background p-8">
    <div class="mx-auto max-w-3xl space-y-8">
      <h1 class="text-2xl font-semibold text-foreground">Meerkat Kitchen Sink</h1>

      <!-- Buttons -->
      <MkCard title="Buttons">
        <div class="flex flex-wrap items-center gap-3">
          <MkButton>Default</MkButton>
          <MkButton variant="secondary">Secondary</MkButton>
          <MkButton variant="outline">Outline</MkButton>
          <MkButton variant="ghost">Ghost</MkButton>
          <MkButton variant="destructive">Destructive</MkButton>
          <MkButton variant="link">Link</MkButton>
          <MkButton disabled>Disabled</MkButton>
        </div>
        <div class="mt-4 flex items-center gap-3">
          <MkButton size="sm">Small</MkButton>
          <MkButton size="default">Default</MkButton>
          <MkButton size="lg">Large</MkButton>
        </div>
      </MkCard>

      <!-- Input -->
      <MkCard title="Input">
        <div class="space-y-3">
          <MkInput v-model="inputValue" placeholder="Type something..." />
          <MkInput placeholder="Disabled" disabled />
          <p class="text-sm text-muted-foreground">Value: {{ inputValue || '(empty)' }}</p>
        </div>
      </MkCard>

      <!-- Badges -->
      <MkCard title="Badges">
        <div class="flex flex-wrap items-center gap-2">
          <MkBadge>Default</MkBadge>
          <MkBadge variant="secondary">Secondary</MkBadge>
          <MkBadge variant="outline">Outline</MkBadge>
          <MkBadge variant="destructive">Destructive</MkBadge>
          <MkBadge variant="success">Success</MkBadge>
          <MkBadge variant="warning">Warning</MkBadge>
          <MkBadge variant="error">Error</MkBadge>
        </div>
      </MkCard>

      <!-- Alerts -->
      <MkCard title="Alerts">
        <div class="space-y-3">
          <MkAlert title="Default">This is a default alert.</MkAlert>
          <MkAlert variant="destructive" title="Error">Something went wrong.</MkAlert>
          <MkAlert variant="success" title="Success">Operation completed.</MkAlert>
          <MkAlert variant="warning" title="Warning">Proceed with caution.</MkAlert>
        </div>
      </MkCard>

      <!-- Spinner -->
      <MkCard title="Spinner">
        <div class="flex items-center gap-4">
          <MkSpinner size="sm" />
          <MkSpinner size="md" />
          <MkSpinner size="lg" />
        </div>
      </MkCard>

      <!-- Toasts -->
      <MkCard title="Toasts">
        <div class="flex flex-wrap items-center gap-3">
          <MkButton variant="outline" size="sm" @click="toast.success('Operation completed successfully.')">Success</MkButton>
          <MkButton variant="outline" size="sm" @click="toast.error('Something went wrong.')">Error</MkButton>
          <MkButton variant="outline" size="sm" @click="toast.info('New deployment detected.')">Info</MkButton>
          <MkButton variant="outline" size="sm" @click="toast.warning('Rate limit approaching.')">Warning</MkButton>
        </div>
      </MkCard>
    </div>
  </div>
</template>
