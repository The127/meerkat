<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkCard, MkButton, MkInput, MkSpinner } from '@/components/meerkat'
import { useOrganization } from '@/composables/useOrganization'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useRenameOrganization } from '@/composables/useRenameOrganization'
import { useDeleteOrganization } from '@/composables/useDeleteOrganization'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'

const toast = useToast()
const { data: org } = useOrganization()
const { canRenameOrg, canDeleteOrg } = useCurrentUser()
const { mutateAsync: renameOrg, isPending: isRenaming } = useRenameOrganization()
const { mutateAsync: deleteOrg, isPending: isDeleting } = useDeleteOrganization()

const name = ref('')
const renameError = ref('')
const showDeleteDialog = ref(false)
const deleteConfirmation = ref('')
const deleteError = ref('')

watch(() => org.value?.name, (val) => {
  if (val) name.value = val
}, { immediate: true })

const nameChanged = ref(false)
watch(name, (val) => {
  nameChanged.value = val.trim() !== (org.value?.name ?? '')
})

async function submitRename() {
  if (!name.value.trim() || !nameChanged.value) return
  renameError.value = ''

  try {
    await renameOrg(name.value.trim())
    toast.success('Organization renamed')
    nameChanged.value = false
  } catch (err) {
    if (err instanceof ApiRequestError) {
      renameError.value = err.error.message
    } else {
      renameError.value = 'An unexpected error occurred'
    }
  }
}

function openDeleteDialog() {
  deleteConfirmation.value = ''
  deleteError.value = ''
  showDeleteDialog.value = true
}

async function submitDelete() {
  if (deleteConfirmation.value !== org.value?.name) return
  deleteError.value = ''

  try {
    await deleteOrg()
  } catch (err) {
    if (err instanceof ApiRequestError) {
      deleteError.value = err.error.message
    } else {
      deleteError.value = 'An unexpected error occurred'
    }
  }
}
</script>

<template>
  <div>
    <h1 class="text-xl font-semibold text-foreground mb-1">Organization Settings</h1>
    <p class="text-sm text-muted-foreground mb-6">Manage your organization.</p>

    <div class="space-y-6 max-w-lg">
      <!-- Rename -->
      <MkCard v-if="canRenameOrg" title="General">
        <form @submit.prevent="submitRename" class="space-y-3">
          <div class="grid gap-1.5">
            <label for="org-name" class="text-sm font-medium text-foreground">Organization name</label>
            <MkInput
              id="org-name"
              v-model="name"
              :disabled="isRenaming"
            />
          </div>
          <p v-if="renameError" class="text-sm text-destructive">{{ renameError }}</p>
          <MkButton type="submit" size="sm" :disabled="isRenaming || !nameChanged || !name.trim()">
            <MkSpinner v-if="isRenaming" size="sm" class="mr-2" />
            Save
          </MkButton>
        </form>
      </MkCard>

      <!-- Danger zone -->
      <MkCard v-if="canDeleteOrg" title="Danger zone" class="border-destructive/50">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-foreground">Delete organization</p>
            <p class="text-xs text-muted-foreground mt-0.5">
              Permanently delete this organization and all its projects.
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
          <DialogTitle>Delete organization</DialogTitle>
          <DialogDescription>
            This will permanently delete <strong>{{ org?.name }}</strong> and all its projects. This action cannot be undone.
          </DialogDescription>
        </DialogHeader>

        <form @submit.prevent="submitDelete" class="grid gap-4 py-2">
          <div class="grid gap-1.5">
            <label for="delete-confirm" class="text-sm font-medium text-foreground">
              Type <strong>{{ org?.name }}</strong> to confirm
            </label>
            <MkInput
              id="delete-confirm"
              v-model="deleteConfirmation"
              :placeholder="org?.name"
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
              :disabled="isDeleting || deleteConfirmation !== org?.name"
            >
              <MkSpinner v-if="isDeleting" size="sm" class="mr-2" />
              Delete organization
            </MkButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  </div>
</template>
