<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { MkButton, MkSpinner, MkBadge, MkPagination, MkBackLink } from '@/components/meerkat'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useIssue } from '@/composables/useIssue'
import { useIssueEvents } from '@/composables/useIssueEvents'
import { useResolveIssue } from '@/composables/useResolveIssue'
import { useReopenIssue } from '@/composables/useReopenIssue'
import { useIgnoreIssue } from '@/composables/useIgnoreIssue'
import { usePagination } from '@/composables/usePagination'
import { useToast } from '@/composables/useToast'
import { useQueryClient } from '@tanstack/vue-query'
import { levelVariant, statusVariant, formatRelativeTime } from '@/lib/issue-utils'
import type { Issue, Event } from '@/lib/types'

const route = useRoute()
const toast = useToast()
const queryClient = useQueryClient()
const { hasProjectPermission } = useCurrentUser()

const slug = computed(() => {
  const param = route.params.slug
  return typeof param === 'string' ? param : undefined
})

const issueNumber = computed(() => {
  const param = route.params.issueNumber
  return typeof param === 'string' ? param : undefined
})

const canWrite = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_write') : false)

// --- Issue ---
const { data: issue, isLoading: isLoadingIssue } = useIssue(slug, issueNumber)

// --- Events ---
const { offset, limit, prevPage, nextPage, pageInfo } = usePagination(20)

const { data: eventsData, isLoading: isLoadingEvents } = useIssueEvents(slug, issueNumber, {
  limit,
  offset,
})

// --- Issue actions ---
const { mutateAsync: resolveIssue } = useResolveIssue()
const { mutateAsync: reopenIssue } = useReopenIssue()
const { mutateAsync: ignoreIssue } = useIgnoreIssue()

async function handleResolve() {
  if (!slug.value || !issueNumber.value || !issue.value) return
  try {
    await resolveIssue({ slug: slug.value, issueNumber: issue.value.issue_number })
    toast.success('Issue resolved')
    queryClient.invalidateQueries({ queryKey: ['issue', slug.value, issueNumber.value] })
  } catch {
    toast.error('Failed to resolve issue')
  }
}

async function handleReopen() {
  if (!slug.value || !issueNumber.value || !issue.value) return
  try {
    await reopenIssue({ slug: slug.value, issueNumber: issue.value.issue_number })
    toast.success('Issue reopened')
    queryClient.invalidateQueries({ queryKey: ['issue', slug.value, issueNumber.value] })
  } catch {
    toast.error('Failed to reopen issue')
  }
}

async function handleIgnore() {
  if (!slug.value || !issueNumber.value || !issue.value) return
  try {
    await ignoreIssue({ slug: slug.value, issueNumber: issue.value.issue_number })
    toast.success('Issue ignored')
    queryClient.invalidateQueries({ queryKey: ['issue', slug.value, issueNumber.value] })
  } catch {
    toast.error('Failed to ignore issue')
  }
}

// --- Extra data expand ---
const expandedEvents = ref<Set<string>>(new Set())

function toggleExtra(eventId: string) {
  if (expandedEvents.value.has(eventId)) {
    expandedEvents.value.delete(eventId)
  } else {
    expandedEvents.value.add(eventId)
  }
}

function hasExtra(extra: unknown): boolean {
  if (!extra) return false
  if (typeof extra === 'object' && Object.keys(extra as object).length === 0) return false
  return true
}

// --- Formatting ---
function formatTimestamp(iso: string): string {
  return new Date(iso).toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

</script>

<template>
  <div>
    <MkBackLink :to="{ name: 'project-issues', params: { slug } }">Back to issues</MkBackLink>

    <!-- Loading -->
    <div v-if="isLoadingIssue" class="flex justify-center py-12">
      <MkSpinner />
    </div>

    <!-- Issue header -->
    <template v-else-if="issue">
      <div class="mb-6">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h1 class="text-xl font-semibold text-foreground break-words">
              <span class="text-muted-foreground font-normal">#{{ issue.issue_number }}</span>
              {{ issue.title }}
            </h1>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <MkBadge :variant="levelVariant(issue.level)">{{ issue.level }}</MkBadge>
              <MkBadge :variant="statusVariant(issue.status)">{{ issue.status }}</MkBadge>
            </div>
          </div>

          <div v-if="canWrite" class="flex items-center gap-1 shrink-0">
            <template v-if="issue.status === 'unresolved'">
              <MkButton size="sm" variant="default" @click="handleResolve">Resolve</MkButton>
              <MkButton size="sm" variant="ghost" @click="handleIgnore">Ignore</MkButton>
            </template>
            <template v-else>
              <MkButton size="sm" variant="outline" @click="handleReopen">Reopen</MkButton>
            </template>
          </div>
        </div>

        <div class="mt-3 flex flex-wrap items-center gap-4 text-xs text-muted-foreground">
          <span>
            <span class="font-medium text-foreground">{{ issue.event_count }}</span>
            {{ issue.event_count === 1 ? 'event' : 'events' }}
          </span>
          <span>First seen {{ formatRelativeTime(issue.first_seen) }}</span>
          <span>Last seen {{ formatRelativeTime(issue.last_seen) }}</span>
        </div>
      </div>

      <!-- Events list -->
      <div>
        <h2 class="text-sm font-medium text-foreground mb-3">Events</h2>

        <div v-if="isLoadingEvents" class="flex justify-center py-8">
          <MkSpinner />
        </div>

        <div v-else-if="!eventsData?.items.length" class="py-8 text-center text-sm text-muted-foreground">
          No events found.
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="event in eventsData.items"
            :key="event.id"
            class="rounded-md border p-3"
          >
            <!-- Event header row -->
            <div class="flex items-center justify-between gap-2 mb-1">
              <div class="flex items-center gap-2 min-w-0">
                <span class="text-xs text-muted-foreground shrink-0">{{ formatTimestamp(event.timestamp) }}</span>
                <MkBadge :variant="levelVariant(event.level)" class="shrink-0">{{ event.level }}</MkBadge>
              </div>
              <span class="text-xs text-muted-foreground shrink-0">{{ event.platform }}</span>
            </div>

            <!-- Message -->
            <p class="text-sm text-foreground mb-1.5">{{ event.message }}</p>

            <!-- Exception -->
            <p v-if="event.exception_type || event.exception_value" class="text-xs text-muted-foreground font-mono mb-1.5">
              <span v-if="event.exception_type" class="font-medium text-foreground">{{ event.exception_type }}</span>
              <span v-if="event.exception_type && event.exception_value">: </span>
              <span v-if="event.exception_value">{{ event.exception_value }}</span>
            </p>

            <!-- Meta badges -->
            <div class="flex flex-wrap items-center gap-1.5 mt-2">
              <span v-if="event.environment" class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-muted text-muted-foreground">
                {{ event.environment }}
              </span>
              <span v-if="event.release" class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-muted text-muted-foreground">
                {{ event.release }}
              </span>
              <span v-if="event.server_name" class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-muted text-muted-foreground">
                {{ event.server_name }}
              </span>
            </div>

            <!-- Tags -->
            <div v-if="event.tags.length" class="flex flex-wrap gap-1.5 mt-2">
              <span
                v-for="tag in event.tags"
                :key="tag.key"
                class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-secondary text-secondary-foreground"
              >
                <span class="font-medium">{{ tag.key }}</span>
                <span class="mx-0.5 text-muted-foreground">:</span>
                {{ tag.value }}
              </span>
            </div>

            <!-- Extra data -->
            <div v-if="hasExtra(event.extra)" class="mt-2">
              <button
                class="text-[11px] text-muted-foreground hover:text-foreground transition-colors"
                @click="toggleExtra(event.id)"
              >
                {{ expandedEvents.has(event.id) ? 'Hide' : 'Show' }} extra data
              </button>
              <pre
                v-if="expandedEvents.has(event.id)"
                class="mt-1.5 rounded bg-muted p-2 text-xs text-muted-foreground font-mono overflow-x-auto max-h-48 overflow-y-auto"
              >{{ JSON.stringify(event.extra, null, 2) }}</pre>
            </div>
          </div>

          <!-- Pagination -->
          <MkPagination
            v-if="eventsData"
            v-bind="pageInfo(eventsData.total)"
            @prev="prevPage"
            @next="nextPage"
          />
        </div>
      </div>
    </template>
  </div>
</template>
