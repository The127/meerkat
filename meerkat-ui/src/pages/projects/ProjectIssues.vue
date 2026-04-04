<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { MkCard, MkButton, MkSpinner, MkBadge } from '@/components/meerkat'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useProjectKeys } from '@/composables/useProjectKeys'
import { useIssues } from '@/composables/useIssues'
import { useToast } from '@/composables/useToast'
import { useQueryClient } from '@tanstack/vue-query'
import type { Issue } from '@/lib/types'

const route = useRoute()
const slug = computed(() => {
  const param = route.params.slug
  return typeof param === 'string' ? param : undefined
})

const toast = useToast()
const queryClient = useQueryClient()
const { hasProjectPermission } = useCurrentUser()

const canManageKeys = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_manage_keys') : false)

// --- Keys (for demo button) ---
const { data: keysData } = useProjectKeys(slug, { status: computed(() => 'active') })
const activeKeys = computed(() => keysData.value?.items ?? [])
const canSendDemo = computed(() => canManageKeys.value && activeKeys.value.length > 0)

// --- Issues ---
const issueStatusFilter = ref<string | undefined>(undefined)
const { data: issuesData, isLoading: isLoadingIssues } = useIssues(slug, {
  status: computed(() => issueStatusFilter.value),
})

// --- Demo event ---
const isSendingDemo = ref(false)

async function sendDemoEvent() {
  const key = activeKeys.value[0]
  if (!key) return
  isSendingDemo.value = true

  try {
    const response = await fetch('/api/v1/ingest', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Meerkat-Key': key.key_token,
      },
      body: JSON.stringify({
        message: "TypeError: Cannot read properties of undefined (reading 'name')",
        level: 'error',
        platform: 'javascript',
        timestamp: new Date().toISOString(),
        server_name: 'web-prod-1',
        environment: 'production',
        release: 'v1.0.0',
        exception_type: 'TypeError',
        exception_value: "Cannot read properties of undefined (reading 'name')",
        tags: [{ key: 'browser', value: 'Chrome 120' }, { key: 'os', value: 'Linux' }],
        extra: { user_id: 'demo-user', url: '/api/users/42' },
      }),
    })

    if (!response.ok) {
      throw new Error(response.statusText)
    }

    toast.success('Demo event sent')
    queryClient.invalidateQueries({ queryKey: ['issues'] })
  } catch {
    toast.error('Failed to send demo event')
  } finally {
    isSendingDemo.value = false
  }
}

function levelVariant(level: Issue['level']): string {
  switch (level) {
    case 'fatal':
    case 'error':
      return 'error'
    case 'warning':
      return 'warning'
    case 'info':
      return 'default'
    case 'debug':
      return 'secondary'
  }
}

function statusVariant(status: Issue['status']): string {
  switch (status) {
    case 'unresolved':
      return 'destructive'
    case 'resolved':
      return 'success'
    case 'ignored':
      return 'secondary'
  }
}

function formatRelativeTime(iso: string): string {
  const now = Date.now()
  const then = new Date(iso).getTime()
  const diffMs = now - then

  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  return new Date(iso).toLocaleDateString()
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-xl font-semibold text-foreground">Issues</h1>
      <p class="text-sm text-muted-foreground">Error events grouped by fingerprint</p>
    </div>

    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex gap-1">
          <MkButton
            v-for="tab in [
              { label: 'All', value: undefined },
              { label: 'Unresolved', value: 'unresolved' },
              { label: 'Resolved', value: 'resolved' },
              { label: 'Ignored', value: 'ignored' },
            ]"
            :key="tab.label"
            size="sm"
            :variant="issueStatusFilter === tab.value ? 'default' : 'ghost'"
            @click="issueStatusFilter = tab.value"
          >
            {{ tab.label }}
          </MkButton>
        </div>

        <MkButton
          v-if="canSendDemo"
          size="sm"
          variant="outline"
          :disabled="isSendingDemo"
          @click="sendDemoEvent"
        >
          <MkSpinner v-if="isSendingDemo" size="sm" class="mr-2" />
          Send demo event
        </MkButton>
      </div>

      <div v-if="isLoadingIssues" class="flex justify-center py-12">
        <MkSpinner />
      </div>

      <div v-else-if="!issuesData?.items.length" class="py-12 text-center text-sm text-muted-foreground">
        No issues yet — send a demo event to get started!
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="issue in issuesData.items"
          :key="issue.id"
          class="rounded-md border p-3"
        >
          <div class="flex items-center justify-between mb-1">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-sm font-medium text-foreground truncate">{{ issue.title }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-2">
              <MkBadge :variant="levelVariant(issue.level)">
                {{ issue.level }}
              </MkBadge>
              <MkBadge :variant="statusVariant(issue.status)">
                {{ issue.status }}
              </MkBadge>
            </div>
          </div>
          <div class="flex items-center gap-3 text-xs text-muted-foreground">
            <span :class="issue.event_count > 10 ? 'font-semibold text-foreground' : ''">
              {{ issue.event_count }} {{ issue.event_count === 1 ? 'event' : 'events' }}
            </span>
            <span>Last seen {{ formatRelativeTime(issue.last_seen) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
