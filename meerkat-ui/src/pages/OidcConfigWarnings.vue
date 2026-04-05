<script setup lang="ts">
import { computed } from 'vue'
import { X } from 'lucide-vue-next'
import { MkBadge, MkButton } from '@/components/meerkat'
import { useOidcConfigWarnings } from '@/composables/useOidcConfigWarnings'
import { useDismissOidcConfigWarning } from '@/composables/useDismissOidcConfigWarning'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'

const props = defineProps<{
  configId: string
}>()

const toast = useToast()
const configIdRef = computed(() => props.configId)
const { data: warnings } = useOidcConfigWarnings(configIdRef)
const { mutateAsync: dismissWarning } = useDismissOidcConfigWarning()

async function handleDismiss(warningKey: string) {
  try {
    await dismissWarning({ configId: props.configId, warningKey })
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  }
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString()
}

function formatContext(context: Record<string, unknown>): string {
  return Object.entries(context)
    .map(([key, value]) => {
      const label = key.replace(/_/g, ' ').replace(/\b\w/, c => c.toUpperCase())
      return `${label}: ${value}`
    })
    .join(' \u00b7 ')
}
</script>

<template>
  <div v-if="warnings?.length" class="space-y-1.5 mt-2 ml-6">
    <div
      v-for="warning in warnings"
      :key="warning.warning_key"
      class="flex items-start justify-between gap-2 rounded border border-warning/30 bg-warning/5 px-3 py-2"
    >
      <div class="min-w-0 flex-1">
        <p class="text-xs text-warning font-medium">{{ warning.message }}</p>
        <p v-if="warning.context" class="text-xs text-muted-foreground mt-0.5">{{ formatContext(warning.context) }}</p>
        <p class="text-xs text-muted-foreground mt-0.5">
          <MkBadge variant="warning" class="mr-1.5">{{ warning.occurrence_count }}x</MkBadge>
          Last seen {{ formatDate(warning.last_seen) }}
        </p>
      </div>
      <MkButton
        size="sm"
        variant="ghost"
        class="shrink-0 h-6 w-6 p-0 text-muted-foreground hover:text-foreground"
        @click="handleDismiss(warning.warning_key)"
      >
        <X class="h-3.5 w-3.5" />
      </MkButton>
    </div>
  </div>
</template>
