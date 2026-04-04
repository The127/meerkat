<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Copy, Check } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkButton, MkInput, MkSpinner, MkBadge } from '@/components/meerkat'
import { useRoute } from 'vue-router'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useProjectKeys } from '@/composables/useProjectKeys'
import { useCreateProjectKey } from '@/composables/useCreateProjectKey'
import { useRevokeProjectKey } from '@/composables/useRevokeProjectKey'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'
import type { ProjectKey } from '@/lib/types'

const route = useRoute()
const slug = computed(() => {
  const param = route.params.slug
  return typeof param === 'string' ? param : undefined
})

const toast = useToast()
const { hasProjectPermission } = useCurrentUser()

const canManageKeys = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_manage_keys') : false)

// --- Project keys ---
const showRevokedKeys = ref(false)
const keyStatus = computed(() => showRevokedKeys.value ? undefined : 'active')
const PAGE_SIZE = 20
const offset = ref(0)
const { data: keysData, isLoading: isLoadingKeys } = useProjectKeys(slug, {
  status: computed(() => keyStatus.value),
  limit: computed(() => PAGE_SIZE),
  offset,
})
const currentPage = computed(() => Math.floor(offset.value / PAGE_SIZE) + 1)
const totalPages = computed(() => Math.ceil((keysData.value?.total ?? 0) / PAGE_SIZE))
const hasPrev = computed(() => offset.value > 0)
const hasNext = computed(() => keysData.value ? offset.value + PAGE_SIZE < keysData.value.total : false)
function prevPage() { offset.value = Math.max(0, offset.value - PAGE_SIZE) }
function nextPage() { offset.value += PAGE_SIZE }
watch(showRevokedKeys, () => { offset.value = 0 })
const { mutateAsync: createKey, isPending: isCreatingKey } = useCreateProjectKey()
const { mutateAsync: revokeKey, isPending: isRevokingKey } = useRevokeProjectKey()

// Create key dialog
const showCreateKeyDialog = ref(false)
const newKeyLabel = ref('')
const createKeyError = ref('')

function openCreateKeyDialog() {
  newKeyLabel.value = ''
  createKeyError.value = ''
  showCreateKeyDialog.value = true
}

async function submitCreateKey() {
  if (!slug.value || !newKeyLabel.value.trim()) return
  createKeyError.value = ''

  try {
    await createKey({ slug: slug.value, label: newKeyLabel.value.trim() })
    toast.success('Key created')
    showCreateKeyDialog.value = false
  } catch (err) {
    if (err instanceof ApiRequestError) {
      createKeyError.value = err.error.message
    } else {
      createKeyError.value = 'An unexpected error occurred'
    }
  }
}

// Revoke key dialog
const showRevokeDialog = ref(false)
const keyToRevoke = ref<ProjectKey | null>(null)
const revokeError = ref('')

function openRevokeDialog(key: ProjectKey) {
  keyToRevoke.value = key
  revokeError.value = ''
  showRevokeDialog.value = true
}

async function submitRevoke() {
  if (!slug.value || !keyToRevoke.value) return
  revokeError.value = ''

  try {
    await revokeKey({ slug: slug.value, keyId: keyToRevoke.value.id })
    toast.success('Key revoked')
    showRevokeDialog.value = false
  } catch (err) {
    if (err instanceof ApiRequestError) {
      revokeError.value = err.error.message
    } else {
      revokeError.value = 'An unexpected error occurred'
    }
  }
}

// Copy DSN to clipboard
const copiedKeyId = ref<string | null>(null)

async function copyDsn(key: ProjectKey) {
  await navigator.clipboard.writeText(key.dsn)
  copiedKeyId.value = key.id
  setTimeout(() => {
    if (copiedKeyId.value === key.id) copiedKeyId.value = null
  }, 2000)
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div>
    <div class="mb-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-semibold text-foreground">Client Keys</h1>
          <p class="text-sm text-muted-foreground">Manage API keys for this project</p>
        </div>
        <MkButton v-if="canManageKeys" size="sm" @click="openCreateKeyDialog">
          Create key
        </MkButton>
      </div>
    </div>

    <div class="space-y-4">
      <label class="flex items-center gap-2 text-sm text-muted-foreground">
        <input
          v-model="showRevokedKeys"
          type="checkbox"
          class="rounded border-input"
        />
        Show revoked keys
      </label>

      <div v-if="isLoadingKeys" class="flex justify-center py-12">
        <MkSpinner />
      </div>

      <div v-else-if="!keysData?.items.length" class="py-12 text-center text-sm text-muted-foreground">
        No keys found
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="key in keysData.items"
          :key="key.id"
          class="rounded-md border p-3"
          :class="key.status === 'revoked' ? 'opacity-50' : ''"
        >
          <div class="flex items-center justify-between mb-1.5">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-foreground">{{ key.label }}</span>
              <MkBadge :variant="key.status === 'active' ? 'success' : 'secondary'">
                {{ key.status === 'active' ? 'Active' : 'Revoked' }}
              </MkBadge>
            </div>
            <MkButton
              v-if="key.status === 'active' && canManageKeys"
              variant="ghost"
              size="sm"
              class="text-destructive hover:text-destructive"
              @click="openRevokeDialog(key)"
            >
              Revoke
            </MkButton>
          </div>

          <div class="flex items-center gap-1.5 mb-1">
            <code class="font-mono text-xs text-muted-foreground break-all">{{ key.dsn }}</code>
            <button
              class="shrink-0 p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
              title="Copy DSN"
              @click="copyDsn(key)"
            >
              <Check v-if="copiedKeyId === key.id" class="h-3.5 w-3.5 text-success" />
              <Copy v-else class="h-3.5 w-3.5" />
            </button>
          </div>

          <p class="text-xs text-muted-foreground">
            Created {{ formatDate(key.created_at) }}
          </p>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="flex items-center justify-between pt-2">
        <span class="text-xs text-muted-foreground">
          Page {{ currentPage }} of {{ totalPages }}
        </span>
        <div class="flex gap-2">
          <MkButton size="sm" variant="outline" :disabled="!hasPrev" @click="prevPage">
            Previous
          </MkButton>
          <MkButton size="sm" variant="outline" :disabled="!hasNext" @click="nextPage">
            Next
          </MkButton>
        </div>
      </div>
    </div>

    <!-- Create key dialog -->
    <Dialog :open="showCreateKeyDialog" @update:open="showCreateKeyDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create client key</DialogTitle>
          <DialogDescription>
            Create a new API key for this project.
          </DialogDescription>
        </DialogHeader>

        <form @submit.prevent="submitCreateKey" class="grid gap-4 py-2">
          <div class="grid gap-1.5">
            <label for="key-label" class="text-sm font-medium text-foreground">Label</label>
            <MkInput
              id="key-label"
              v-model="newKeyLabel"
              placeholder="e.g. Production, Staging"
              :disabled="isCreatingKey"
            />
          </div>

          <p v-if="createKeyError" class="text-sm text-destructive">{{ createKeyError }}</p>

          <DialogFooter>
            <MkButton variant="outline" type="button" :disabled="isCreatingKey" @click="showCreateKeyDialog = false">
              Cancel
            </MkButton>
            <MkButton type="submit" :disabled="isCreatingKey || !newKeyLabel.trim()">
              <MkSpinner v-if="isCreatingKey" size="sm" class="mr-2" />
              Create key
            </MkButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <!-- Revoke key dialog -->
    <Dialog :open="showRevokeDialog" @update:open="showRevokeDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Revoke client key</DialogTitle>
          <DialogDescription>
            Are you sure you want to revoke <strong>{{ keyToRevoke?.label }}</strong>? Applications using this key will no longer be able to send data.
          </DialogDescription>
        </DialogHeader>

        <p v-if="revokeError" class="text-sm text-destructive">{{ revokeError }}</p>

        <DialogFooter>
          <MkButton variant="outline" :disabled="isRevokingKey" @click="showRevokeDialog = false">
            Cancel
          </MkButton>
          <MkButton variant="destructive" :disabled="isRevokingKey" @click="submitRevoke">
            <MkSpinner v-if="isRevokingKey" size="sm" class="mr-2" />
            Revoke key
          </MkButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
