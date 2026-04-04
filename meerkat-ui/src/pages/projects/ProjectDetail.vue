<script setup lang="ts">
import { ref, computed } from 'vue'
import { Copy, Check } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkCard, MkButton, MkInput, MkSpinner, MkBadge, MkEditableText } from '@/components/meerkat'
import { useCurrentProject } from '@/composables/useCurrentProject'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useRenameProject } from '@/composables/useRenameProject'
import { useDeleteProject } from '@/composables/useDeleteProject'
import { useProjectKeys } from '@/composables/useProjectKeys'
import { useCreateProjectKey } from '@/composables/useCreateProjectKey'
import { useRevokeProjectKey } from '@/composables/useRevokeProjectKey'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'
import type { ProjectKey } from '@/lib/types'

const toast = useToast()
const { currentProject, slug, isLoading } = useCurrentProject()
const { hasProjectPermission } = useCurrentUser()
const { mutateAsync: renameProject } = useRenameProject()
const { mutateAsync: deleteProject, isPending: isDeleting } = useDeleteProject()

const canWrite = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_write') : false)
const canDelete = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_delete') : false)
const canManageKeys = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_manage_keys') : false)
const canReadProject = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_read') : false)

// --- Project keys ---
const showRevokedKeys = ref(false)
const keyStatus = computed(() => showRevokedKeys.value ? undefined : 'active')
const { data: keysData, isLoading: isLoadingKeys } = useProjectKeys(slug, { status: computed(() => keyStatus.value) })
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

// --- Project rename ---
async function handleRename(newName: string) {
  if (!slug.value) return
  try {
    await renameProject({ slug: slug.value, name: newName })
    toast.success('Project renamed')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  }
}

// --- Project delete ---
const showDeleteDialog = ref(false)
const deleteConfirmation = ref('')
const deleteError = ref('')

function openDeleteDialog() {
  deleteConfirmation.value = ''
  deleteError.value = ''
  showDeleteDialog.value = true
}

async function submitDelete() {
  if (!slug.value || deleteConfirmation.value !== currentProject.value?.name) return
  deleteError.value = ''

  try {
    await deleteProject(slug.value)
    toast.success('Project deleted')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      deleteError.value = err.error.message
    } else {
      deleteError.value = 'An unexpected error occurred'
    }
  }
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
      <div class="flex items-center gap-3 mb-1">
        <MkEditableText
          v-if="currentProject"
          :model-value="currentProject.name"
          :disabled="!canWrite"
          class="text-xl font-semibold text-foreground"
          @save="handleRename"
        />
        <h1 v-else class="text-xl font-semibold text-foreground">{{ slug }}</h1>
        <MkBadge v-if="currentProject" variant="secondary">{{ currentProject.slug }}</MkBadge>
      </div>
      <p v-if="currentProject" class="text-sm text-muted-foreground">
        Created {{ formatDate(currentProject.created_at) }}
      </p>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <MkSpinner />
    </div>

    <div v-else-if="currentProject" class="space-y-6 max-w-lg">
      <!-- Client keys -->
      <MkCard v-if="canReadProject">
        <template #header>
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-lg font-semibold leading-none tracking-tight">Client Keys</h3>
              <p class="text-sm text-muted-foreground mt-1">Manage API keys for this project</p>
            </div>
            <MkButton v-if="canManageKeys" size="sm" @click="openCreateKeyDialog">
              Create key
            </MkButton>
          </div>
        </template>

        <div class="space-y-4">
          <label class="flex items-center gap-2 text-sm text-muted-foreground">
            <input
              v-model="showRevokedKeys"
              type="checkbox"
              class="rounded border-input"
            />
            Show revoked keys
          </label>

          <div v-if="isLoadingKeys" class="flex justify-center py-6">
            <MkSpinner />
          </div>

          <div v-else-if="!keysData?.items.length" class="py-6 text-center text-sm text-muted-foreground">
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
        </div>
      </MkCard>

      <!-- Danger zone -->
      <MkCard v-if="canDelete" title="Danger zone" class="border-destructive/50">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-foreground">Delete project</p>
            <p class="text-xs text-muted-foreground mt-0.5">
              Permanently delete this project and all its data.
            </p>
          </div>
          <MkButton variant="destructive" size="sm" @click="openDeleteDialog">
            Delete
          </MkButton>
        </div>
      </MkCard>
    </div>

    <!-- Create key dialog -->
    <Dialog :open="showCreateKeyDialog" @update:open="showCreateKeyDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create client key</DialogTitle>
          <DialogDescription>
            Create a new API key for <strong>{{ currentProject?.name }}</strong>.
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

    <!-- Delete confirmation dialog -->
    <Dialog :open="showDeleteDialog" @update:open="showDeleteDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete project</DialogTitle>
          <DialogDescription>
            This will permanently delete <strong>{{ currentProject?.name }}</strong> and all its data. This action cannot be undone.
          </DialogDescription>
        </DialogHeader>

        <form @submit.prevent="submitDelete" class="grid gap-4 py-2">
          <div class="grid gap-1.5">
            <label for="delete-project-confirm" class="text-sm font-medium text-foreground">
              Type <strong>{{ currentProject?.name }}</strong> to confirm
            </label>
            <MkInput
              id="delete-project-confirm"
              v-model="deleteConfirmation"
              :placeholder="currentProject?.name"
              :disabled="isDeleting"
            />
          </div>

          <p v-if="deleteError" class="text-sm text-destructive">{{ deleteError }}</p>

          <DialogFooter>
            <MkButton variant="outline" type="button" :disabled="isDeleting" @click="showDeleteDialog = false">
              Cancel
            </MkButton>
            <MkButton
              variant="destructive"
              type="submit"
              :disabled="isDeleting || deleteConfirmation !== currentProject?.name"
            >
              <MkSpinner v-if="isDeleting" size="sm" class="mr-2" />
              Delete project
            </MkButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  </div>
</template>
