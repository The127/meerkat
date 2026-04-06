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
import { MkButton, MkInput, MkBadge, MkSpinner, MkEmptyState } from '@/components/meerkat'
import { useCurrentProject } from '@/composables/useCurrentProject'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useProjectRoles } from '@/composables/useProjectRoles'
import { useCreateProjectRole } from '@/composables/useCreateProjectRole'
import { useUpdateProjectRole } from '@/composables/useUpdateProjectRole'
import { useDeleteProjectRole } from '@/composables/useDeleteProjectRole'
import { useToast } from '@/composables/useToast'
import { formatPermission } from '@/lib/member-utils'
import { ApiRequestError } from '@/lib/api'
import type { ProjectRole } from '@/lib/types'

const ALL_PERMISSIONS = [
  'project_read',
  'project_write',
  'project_delete',
  'project_manage_members',
  'project_manage_keys',
]

const toast = useToast()
const { slug, isLoading: isProjectLoading } = useCurrentProject()
const { hasProjectPermission } = useCurrentUser()

const canManageMembers = computed(() =>
  slug.value ? hasProjectPermission(slug.value, 'project_manage_members') : false
)

const { data: roles, isLoading: isRolesLoading } = useProjectRoles(slug)
const { mutateAsync: createRole, isPending: isCreating } = useCreateProjectRole()
const { mutateAsync: updateRole, isPending: isUpdating } = useUpdateProjectRole()
const { mutateAsync: deleteRole, isPending: isDeleting } = useDeleteProjectRole()

const isLoading = computed(() => isProjectLoading.value || isRolesLoading.value)

// --- Create dialog ---
const showCreateDialog = ref(false)
const createName = ref('')
const createPermissions = ref<string[]>([])
const createError = ref('')

function openCreateDialog() {
  createName.value = ''
  createPermissions.value = []
  createError.value = ''
  showCreateDialog.value = true
}

const createFormValid = computed(
  () => createName.value.trim().length > 0 && createPermissions.value.length > 0
)

async function submitCreate() {
  if (!slug.value || !createFormValid.value) return
  createError.value = ''
  try {
    await createRole({ slug: slug.value, name: createName.value.trim(), permissions: createPermissions.value })
    toast.success('Role created')
    showCreateDialog.value = false
  } catch (err) {
    if (err instanceof ApiRequestError) {
      createError.value = err.error.message
    } else {
      createError.value = 'An unexpected error occurred'
    }
  }
}

// --- Edit dialog ---
const showEditDialog = ref(false)
const editRole = ref<ProjectRole | null>(null)
const editName = ref('')
const editPermissions = ref<string[]>([])
const editError = ref('')

function openEditDialog(role: ProjectRole) {
  editRole.value = role
  editName.value = role.name
  editPermissions.value = [...role.permissions]
  editError.value = ''
  showEditDialog.value = true
}

const editFormValid = computed(
  () => editName.value.trim().length > 0 && editPermissions.value.length > 0
)

async function submitEdit() {
  if (!slug.value || !editRole.value || !editFormValid.value) return
  editError.value = ''
  try {
    await updateRole({
      slug: slug.value,
      roleId: editRole.value.id,
      name: editName.value.trim(),
      permissions: editPermissions.value,
    })
    toast.success('Role updated')
    showEditDialog.value = false
  } catch (err) {
    if (err instanceof ApiRequestError) {
      editError.value = err.error.message
    } else {
      editError.value = 'An unexpected error occurred'
    }
  }
}

// --- Delete dialog ---
const showDeleteDialog = ref(false)
const deleteTarget = ref<ProjectRole | null>(null)
const deleteError = ref('')

function openDeleteDialog(role: ProjectRole) {
  deleteTarget.value = role
  deleteError.value = ''
  showDeleteDialog.value = true
}

async function submitDelete() {
  if (!slug.value || !deleteTarget.value) return
  deleteError.value = ''
  try {
    await deleteRole({ slug: slug.value, roleId: deleteTarget.value.id })
    toast.success('Role deleted')
    showDeleteDialog.value = false
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
    <div class="mb-6 flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-foreground">Roles</h1>
        <p class="text-sm text-muted-foreground">Manage project roles and their permissions</p>
      </div>
      <MkButton v-if="canManageMembers" size="sm" @click="openCreateDialog">
        Create role
      </MkButton>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <MkSpinner />
    </div>

    <MkEmptyState
      v-else-if="!roles?.length"
      title="No roles"
      description="No project roles have been created yet."
    />

    <div v-else class="rounded-md border overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b bg-muted/40">
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Name</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Slug</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Permissions</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Default</th>
            <th v-if="canManageMembers" class="px-4 py-2.5 text-right font-medium text-muted-foreground">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="role in roles"
            :key="role.id"
            class="border-b last:border-0 hover:bg-muted/20 transition-colors"
          >
            <td class="px-4 py-3 font-medium text-foreground">{{ role.name }}</td>
            <td class="px-4 py-3 text-muted-foreground font-mono text-xs">{{ role.slug }}</td>
            <td class="px-4 py-3">
              <div class="flex flex-wrap gap-1">
                <MkBadge
                  v-for="perm in role.permissions"
                  :key="perm"
                  variant="secondary"
                >
                  {{ formatPermission(perm) }}
                </MkBadge>
              </div>
            </td>
            <td class="px-4 py-3">
              <MkBadge v-if="role.is_default" variant="outline">Default</MkBadge>
            </td>
            <td v-if="canManageMembers" class="px-4 py-3 text-right">
              <div class="flex items-center justify-end gap-2">
                <MkButton size="sm" variant="outline" @click="openEditDialog(role)">
                  Edit
                </MkButton>
                <MkButton size="sm" variant="destructive" @click="openDeleteDialog(role)">
                  Delete
                </MkButton>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Create dialog -->
    <Dialog :open="showCreateDialog" @update:open="showCreateDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create role</DialogTitle>
          <DialogDescription>
            Define a new role with a name and set of permissions.
          </DialogDescription>
        </DialogHeader>

        <form @submit.prevent="submitCreate" class="grid gap-4 py-2">
          <div class="grid gap-1.5">
            <label for="create-role-name" class="text-sm font-medium text-foreground">Name</label>
            <MkInput
              id="create-role-name"
              v-model="createName"
              placeholder="e.g. Viewer"
              :disabled="isCreating"
            />
          </div>

          <div class="grid gap-2">
            <span class="text-sm font-medium text-foreground">Permissions</span>
            <div class="grid gap-1.5">
              <label
                v-for="perm in ALL_PERMISSIONS"
                :key="perm"
                class="flex items-center gap-2 text-sm text-foreground cursor-pointer"
              >
                <input
                  type="checkbox"
                  :value="perm"
                  v-model="createPermissions"
                  :disabled="isCreating"
                  class="rounded border-border"
                />
                {{ formatPermission(perm) }}
              </label>
            </div>
          </div>

          <p v-if="createError" class="text-sm text-destructive">{{ createError }}</p>

          <DialogFooter>
            <MkButton variant="outline" type="button" :disabled="isCreating" @click="showCreateDialog = false">
              Cancel
            </MkButton>
            <MkButton type="submit" :disabled="isCreating || !createFormValid">
              <MkSpinner v-if="isCreating" size="sm" class="mr-2" />
              Create role
            </MkButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <!-- Edit dialog -->
    <Dialog :open="showEditDialog" @update:open="showEditDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit role</DialogTitle>
          <DialogDescription>
            Update the name and permissions for <strong>{{ editRole?.name }}</strong>.
          </DialogDescription>
        </DialogHeader>

        <form @submit.prevent="submitEdit" class="grid gap-4 py-2">
          <div class="grid gap-1.5">
            <label for="edit-role-name" class="text-sm font-medium text-foreground">Name</label>
            <MkInput
              id="edit-role-name"
              v-model="editName"
              placeholder="Role name"
              :disabled="isUpdating"
            />
          </div>

          <div class="grid gap-2">
            <span class="text-sm font-medium text-foreground">Permissions</span>
            <div class="grid gap-1.5">
              <label
                v-for="perm in ALL_PERMISSIONS"
                :key="perm"
                class="flex items-center gap-2 text-sm text-foreground cursor-pointer"
              >
                <input
                  type="checkbox"
                  :value="perm"
                  v-model="editPermissions"
                  :disabled="isUpdating"
                  class="rounded border-border"
                />
                {{ formatPermission(perm) }}
              </label>
            </div>
          </div>

          <p v-if="editError" class="text-sm text-destructive">{{ editError }}</p>

          <DialogFooter>
            <MkButton variant="outline" type="button" :disabled="isUpdating" @click="showEditDialog = false">
              Cancel
            </MkButton>
            <MkButton type="submit" :disabled="isUpdating || !editFormValid">
              <MkSpinner v-if="isUpdating" size="sm" class="mr-2" />
              Save changes
            </MkButton>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <!-- Delete confirmation dialog -->
    <Dialog :open="showDeleteDialog" @update:open="showDeleteDialog = $event">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete role</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete <strong>{{ deleteTarget?.name }}</strong>? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>

        <p v-if="deleteError" class="text-sm text-destructive">{{ deleteError }}</p>

        <DialogFooter>
          <MkButton variant="outline" :disabled="isDeleting" @click="showDeleteDialog = false">
            Cancel
          </MkButton>
          <MkButton variant="destructive" :disabled="isDeleting" @click="submitDelete">
            <MkSpinner v-if="isDeleting" size="sm" class="mr-2" />
            Delete role
          </MkButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
