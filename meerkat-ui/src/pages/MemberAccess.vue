<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Shield, FolderOpen } from 'lucide-vue-next'
import { MkSpinner, MkBadge, MkEmptyState, MkBackLink } from '@/components/meerkat'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useMemberAccess } from '@/composables/useMemberAccess'
import { formatRelativeTime, formatDate } from '@/lib/date-utils'
import { roleVariant, formatPermission } from '@/lib/member-utils'

const route = useRoute()
const router = useRouter()

const memberId = computed(() => {
  const param = route.params.id
  return typeof param === 'string' ? param : undefined
})

const { canManageMembers } = useCurrentUser()
const { data: access, isLoading } = useMemberAccess(memberId)
</script>

<template>
  <div v-if="!canManageMembers" class="py-12 text-center text-sm text-muted-foreground">
    You don't have permission to view member details.
  </div>
  <div v-else>
    <MkBackLink :to="{ name: 'members' }">Back to members</MkBackLink>

    <!-- Loading -->
    <div v-if="isLoading" class="flex justify-center py-12">
      <MkSpinner />
    </div>

    <template v-else-if="access">
      <!-- Header -->
      <div class="mb-6">
        <h1 class="text-xl font-semibold text-foreground">{{ access.preferred_name }}</h1>
        <p class="text-sm text-muted-foreground">{{ access.sub }}</p>
        <div class="mt-2 flex items-center gap-4 text-xs text-muted-foreground">
          <span>Joined {{ formatDate(access.created_at) }}</span>
          <span>Last seen {{ formatRelativeTime(access.last_seen) }}</span>
        </div>
      </div>

      <!-- Org access -->
      <div class="mb-6">
        <h2 class="text-sm font-medium text-foreground mb-3 flex items-center gap-1.5">
          <Shield class="w-4 h-4 text-muted-foreground" />
          Organization Access
        </h2>
        <div class="rounded-md border p-4">
          <div class="flex flex-wrap items-center gap-2 mb-3">
            <MkBadge
              v-for="role in access.org_access.roles"
              :key="role"
              :variant="roleVariant(role)"
            >
              {{ role }}
            </MkBadge>
          </div>
          <div v-if="access.org_access.permissions.length" class="space-y-1">
            <p class="text-xs font-medium text-muted-foreground mb-1.5">Permissions</p>
            <div class="flex flex-wrap gap-1.5">
              <span
                v-for="perm in access.org_access.permissions"
                :key="perm"
                class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-muted text-muted-foreground"
              >
                {{ formatPermission(perm) }}
              </span>
            </div>
          </div>
          <p v-else class="text-xs text-muted-foreground">No organization permissions</p>
        </div>
      </div>

      <!-- Project access -->
      <div>
        <h2 class="text-sm font-medium text-foreground mb-3 flex items-center gap-1.5">
          <FolderOpen class="w-4 h-4 text-muted-foreground" />
          Project Access
        </h2>

        <MkEmptyState
          v-if="!access.project_access.length"
          :icon="FolderOpen"
          title="No project access"
          description="This member is not assigned to any projects."
        />

        <div v-else class="space-y-3">
          <div
            v-for="project in access.project_access"
            :key="project.project_slug"
            class="rounded-md border p-4"
          >
            <div class="mb-3">
              <button
                class="text-sm font-medium text-foreground hover:text-primary transition-colors cursor-pointer"
                @click="router.push({ name: 'project-issues', params: { slug: project.project_slug } })"
              >
                {{ project.project_name }}
              </button>
            </div>

            <div class="space-y-3">
              <div
                v-for="role in project.roles"
                :key="role.role_name"
              >
                <div class="flex items-center gap-2 mb-1.5">
                  <MkBadge variant="outline">{{ role.role_name }}</MkBadge>
                </div>
                <div class="flex flex-wrap gap-1.5 pl-1">
                  <span
                    v-for="perm in role.permissions"
                    :key="perm"
                    class="inline-flex items-center rounded px-1.5 py-0.5 text-[11px] bg-muted text-muted-foreground"
                  >
                    {{ formatPermission(perm) }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
