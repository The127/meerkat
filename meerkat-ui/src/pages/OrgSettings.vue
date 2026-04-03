<script setup lang="ts">
import { ref, watch } from 'vue'
import { Pencil, Plus, Shield, ShieldCheck, ShieldOff } from 'lucide-vue-next'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkCard, MkButton, MkInput, MkSpinner, MkBadge } from '@/components/meerkat'
import { useOrganization } from '@/composables/useOrganization'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useRenameOrganization } from '@/composables/useRenameOrganization'
import { useDeleteOrganization } from '@/composables/useDeleteOrganization'
import { useOidcConfigs } from '@/composables/useOidcConfigs'
import { useActivateOidcConfig } from '@/composables/useActivateOidcConfig'
import { useDeleteOidcConfig } from '@/composables/useDeleteOidcConfig'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'
import type { OidcConfigListItem } from '@/lib/types'
import AddOidcConfigDialog from './AddOidcConfigDialog.vue'
import EditOidcClaimMappingDialog from './EditOidcClaimMappingDialog.vue'

const toast = useToast()
const { data: org } = useOrganization()
const { canRenameOrg, canDeleteOrg, canManageOidc } = useCurrentUser()
const { mutateAsync: renameOrg, isPending: isRenaming } = useRenameOrganization()
const { mutateAsync: deleteOrg, isPending: isDeleting } = useDeleteOrganization()
const { data: oidcConfigs } = useOidcConfigs()
const { mutateAsync: activateConfig } = useActivateOidcConfig()
const { mutateAsync: deleteConfig } = useDeleteOidcConfig()

// --- Rename ---
const name = ref('')
const renameError = ref('')

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

// --- Delete org ---
const showDeleteDialog = ref(false)
const deleteConfirmation = ref('')
const deleteError = ref('')

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

// --- OIDC configs ---
const showAddOidcDialog = ref(false)
const showEditClaimMappingDialog = ref(false)
const editingConfig = ref<OidcConfigListItem | null>(null)

function openEditClaimMapping(config: OidcConfigListItem) {
  editingConfig.value = config
  showEditClaimMappingDialog.value = true
}

async function handleActivateConfig(configId: string) {
  try {
    await activateConfig(configId)
    toast.success('OIDC configuration activated')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  }
}

async function handleDeleteConfig(configId: string) {
  try {
    await deleteConfig(configId)
    toast.success('OIDC configuration deleted')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  }
}

function statusBadgeVariant(status: string) {
  switch (status) {
    case 'active': return 'success'
    case 'draft': return 'secondary'
    case 'inactive': return 'warning'
    default: return 'secondary'
  }
}

function statusIcon(status: string) {
  switch (status) {
    case 'active': return ShieldCheck
    case 'inactive': return ShieldOff
    default: return Shield
  }
}
</script>

<template>
  <div>
    <h1 class="text-xl font-semibold text-foreground mb-1">Organization Settings</h1>
    <p class="text-sm text-muted-foreground mb-6">Manage your organization.</p>

    <div class="space-y-6 max-w-2xl">
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

      <!-- OIDC Configurations -->
      <MkCard v-if="canManageOidc" title="OIDC Configurations" description="Manage identity provider configurations.">
        <div class="space-y-3">
          <div
            v-for="config in oidcConfigs"
            :key="config.id"
            class="flex items-center justify-between rounded-md border border-border px-4 py-3"
          >
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <component :is="statusIcon(config.status)" class="w-4 h-4 shrink-0 text-muted-foreground" />
                <p class="text-sm font-medium text-foreground truncate">{{ config.name }}</p>
                <MkBadge :variant="statusBadgeVariant(config.status)">{{ config.status }}</MkBadge>
              </div>
              <p class="text-xs text-muted-foreground mt-0.5 ml-6 truncate">{{ config.issuer_url }}</p>
            </div>
            <div class="flex items-center gap-1.5 shrink-0 ml-4">
              <MkButton size="sm" variant="outline" @click="openEditClaimMapping(config)">
                <Pencil class="h-3.5 w-3.5" />
              </MkButton>
              <template v-if="config.status !== 'active'">
                <MkButton size="sm" variant="outline" @click="handleActivateConfig(config.id)">Activate</MkButton>
                <MkButton size="sm" variant="destructive" @click="handleDeleteConfig(config.id)">Delete</MkButton>
              </template>
            </div>
          </div>

          <MkButton size="sm" variant="outline" @click="showAddOidcDialog = true">
            <Plus class="h-4 w-4 mr-1.5" />
            Add configuration
          </MkButton>
        </div>
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

    <!-- Add OIDC config dialog -->
    <AddOidcConfigDialog v-model:open="showAddOidcDialog" />

    <!-- Edit claim mapping dialog -->
    <EditOidcClaimMappingDialog v-model:open="showEditClaimMappingDialog" :config="editingConfig" />
  </div>
</template>
