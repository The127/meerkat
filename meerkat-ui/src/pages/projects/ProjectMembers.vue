<script setup lang="ts">
import { ref, computed } from 'vue'
import { MkBadge, MkSpinner, MkEmptyState } from '@/components/meerkat'
import { useCurrentProject } from '@/composables/useCurrentProject'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useProjectRoles } from '@/composables/useProjectRoles'
import { useProjectMembers } from '@/composables/useProjectMembers'
import { useAssignRole } from '@/composables/useAssignRole'
import { useRemoveRole } from '@/composables/useRemoveRole'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'
import { formatDate } from '@/lib/date-utils'
import type { ProjectMember, ProjectRole } from '@/lib/types'

const toast = useToast()
const { slug, isLoading: isProjectLoading } = useCurrentProject()
const { hasProjectPermission } = useCurrentUser()

const canManageMembers = computed(() =>
  slug.value ? hasProjectPermission(slug.value, 'project_manage_members') : false
)

const { data: members, isLoading: isMembersLoading } = useProjectMembers(slug)
const { data: roles, isLoading: isRolesLoading } = useProjectRoles(slug)
const { mutateAsync: assignRole } = useAssignRole()
const { mutateAsync: removeRole } = useRemoveRole()

const isLoading = computed(() => isProjectLoading.value || isMembersLoading.value || isRolesLoading.value)

// Track which member is currently having a role assigned (for pending state per member)
const assigningFor = ref<string | null>(null)
const removingKey = ref<string | null>(null)

const unassignedRolesMap = computed(() => {
  if (!roles.value || !members.value) return new Map<string, ProjectRole[]>()
  return new Map(
    members.value.map((m) => {
      const assignedIds = new Set(m.roles.map((r) => r.role_id))
      return [m.member_id, roles.value!.filter((r) => !assignedIds.has(r.id))]
    })
  )
})

async function handleAssignRole(member: ProjectMember, roleId: string) {
  if (!slug.value) return
  assigningFor.value = member.member_id
  try {
    await assignRole({ slug: slug.value, memberId: member.member_id, roleId })
    toast.success('Role assigned')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  } finally {
    assigningFor.value = null
  }
}

async function handleRemoveRole(member: ProjectMember, roleId: string) {
  if (!slug.value) return
  const key = `${member.member_id}:${roleId}`
  removingKey.value = key
  try {
    await removeRole({ slug: slug.value, memberId: member.member_id, roleId })
    toast.success('Role removed')
  } catch (err) {
    if (err instanceof ApiRequestError) {
      toast.error(err.error.message)
    } else {
      toast.error('An unexpected error occurred')
    }
  } finally {
    removingKey.value = null
  }
}
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-xl font-semibold text-foreground">Members</h1>
      <p class="text-sm text-muted-foreground">
        View project members and manage their roles
        <span v-if="members" class="text-muted-foreground">
          &middot; {{ members.length }} {{ members.length === 1 ? 'member' : 'members' }}
        </span>
      </p>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <MkSpinner />
    </div>

    <MkEmptyState
      v-else-if="!members?.length"
      title="No members"
      description="No members have been added to this project yet."
    />

    <div v-else class="rounded-md border overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b bg-muted/40">
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Name</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Subject</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Roles</th>
            <th class="px-4 py-2.5 text-left font-medium text-muted-foreground">Joined</th>
            <th v-if="canManageMembers" class="px-4 py-2.5 text-left font-medium text-muted-foreground">Assign role</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="member in members"
            :key="member.member_id"
            class="border-b last:border-0 hover:bg-muted/20 transition-colors"
          >
            <td class="px-4 py-3 font-medium text-foreground">{{ member.preferred_name }}</td>
            <td class="px-4 py-3 text-muted-foreground font-mono text-xs truncate max-w-[180px]">
              {{ member.sub }}
            </td>
            <td class="px-4 py-3">
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="role in member.roles"
                  :key="role.role_id"
                  class="inline-flex items-center gap-1"
                >
                  <MkBadge variant="secondary">
                    {{ role.role_name }}
                    <button
                      v-if="canManageMembers"
                      type="button"
                      class="ml-1 text-muted-foreground hover:text-foreground transition-colors leading-none"
                      :disabled="removingKey === `${member.member_id}:${role.role_id}`"
                      :aria-label="`Remove ${role.role_name}`"
                      @click="handleRemoveRole(member, role.role_id)"
                    >
                      <MkSpinner
                        v-if="removingKey === `${member.member_id}:${role.role_id}`"
                        size="sm"
                      />
                      <span v-else aria-hidden="true">&times;</span>
                    </button>
                  </MkBadge>
                </span>
                <span v-if="!member.roles.length" class="text-xs text-muted-foreground">None</span>
              </div>
            </td>
            <td class="px-4 py-3 text-muted-foreground text-xs whitespace-nowrap">
              {{ formatDate(member.created_at) }}
            </td>
            <td v-if="canManageMembers" class="px-4 py-3">
              <div class="flex items-center gap-2">
                <select
                  class="text-sm rounded-md border border-border bg-background px-2 py-1 text-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:opacity-50"
                  :disabled="assigningFor === member.member_id || !(unassignedRolesMap.get(member.member_id) ?? []).length"
                  @change="(e) => {
                    const val = (e.target as HTMLSelectElement).value
                    if (val) {
                      handleAssignRole(member, val);
                      (e.target as HTMLSelectElement).value = ''
                    }
                  }"
                >
                  <option value="">
                    {{ (unassignedRolesMap.get(member.member_id) ?? []).length ? 'Assign role...' : 'All roles assigned' }}
                  </option>
                  <option
                    v-for="role in unassignedRolesMap.get(member.member_id) ?? []"
                    :key="role.id"
                    :value="role.id"
                  >
                    {{ role.name }}
                  </option>
                </select>
                <MkSpinner v-if="assigningFor === member.member_id" size="sm" />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
