<script setup lang="ts">
import { ref, computed } from 'vue'
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
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'

const toast = useToast()
const { currentProject, slug, isLoading } = useCurrentProject()
const { hasProjectPermission } = useCurrentUser()
const { mutateAsync: renameProject } = useRenameProject()
const { mutateAsync: deleteProject, isPending: isDeleting } = useDeleteProject()

const canWrite = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_write') : false)
const canDelete = computed(() => slug.value ? hasProjectPermission(slug.value, 'project_delete') : false)

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
