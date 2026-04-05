<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { Users } from 'lucide-vue-next'
import { MkButton, MkSpinner, MkBadge, MkPagination, MkInput, MkEmptyState } from '@/components/meerkat'
import { useCurrentUser } from '@/composables/useCurrentUser'
import { useMembers } from '@/composables/useMembers'
import { usePagination } from '@/composables/usePagination'
import { formatRelativeTime, formatDate } from '@/lib/date-utils'
import { roleVariant } from '@/lib/member-utils'

const router = useRouter()
const { canManageMembers } = useCurrentUser()

const searchInput = ref('')
const searchQuery = ref<string | undefined>(undefined)
const roleFilter = ref<string | undefined>(undefined)

let debounceTimer: ReturnType<typeof setTimeout> | undefined
watch(searchInput, (value) => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    searchQuery.value = value.trim() || undefined
    reset()
  }, 300)
})

const { offset, limit, prevPage, nextPage, reset, pageInfo } = usePagination(20)

watch(roleFilter, () => reset())

const { data, isLoading } = useMembers({
  search: searchQuery,
  role: roleFilter,
  limit,
  offset,
})

</script>

<template>
  <div v-if="!canManageMembers" class="py-12 text-center text-sm text-muted-foreground">
    You don't have permission to view members.
  </div>
  <div v-else>
    <div class="mb-6">
      <h1 class="text-xl font-semibold text-foreground">Members</h1>
      <p class="text-sm text-muted-foreground">
        View organization members and their access
        <span v-if="data" class="text-muted-foreground">&middot; {{ data.total }} {{ data.total === 1 ? 'member' : 'members' }}</span>
      </p>
    </div>

    <div class="space-y-4">
      <!-- Search + filters -->
      <div class="flex items-center gap-3">
        <MkInput
          v-model="searchInput"
          placeholder="Search by name or subject..."
          class="max-w-xs"
        />
        <div class="flex gap-1">
          <MkButton
            v-for="tab in [
              { label: 'All', value: undefined },
              { label: 'Owner', value: 'owner' },
              { label: 'Admin', value: 'admin' },
              { label: 'Member', value: 'member' },
            ]"
            :key="tab.label"
            size="sm"
            :variant="roleFilter === tab.value ? 'default' : 'ghost'"
            @click="roleFilter = tab.value"
          >
            {{ tab.label }}
          </MkButton>
        </div>
      </div>

      <!-- Loading -->
      <div v-if="isLoading" class="flex justify-center py-12">
        <MkSpinner />
      </div>

      <!-- Empty state -->
      <MkEmptyState
        v-else-if="!data?.items.length"
        :icon="Users"
        title="No members found"
        description="Try adjusting your search or filters."
      />

      <!-- Member list -->
      <div v-else class="space-y-2">
        <div
          v-for="member in data.items"
          :key="member.id"
          class="rounded-md border p-3 hover:bg-muted/30 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div class="min-w-0">
              <button
                class="text-sm font-medium text-foreground hover:text-primary transition-colors text-left cursor-pointer"
                @click="router.push({ name: 'member-access', params: { id: member.id } })"
              >
                {{ member.preferred_name }}
              </button>
              <p class="text-xs text-muted-foreground truncate">{{ member.sub }}</p>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-4">
              <MkBadge
                v-for="role in member.org_roles"
                :key="role"
                :variant="roleVariant(role)"
              >
                {{ role }}
              </MkBadge>
            </div>
          </div>
          <div class="flex items-center gap-4 mt-1.5 text-xs text-muted-foreground">
            <span>Last seen {{ formatRelativeTime(member.last_seen) }}</span>
            <span>Joined {{ formatDate(member.created_at) }}</span>
          </div>
        </div>

        <!-- Pagination -->
        <MkPagination
          v-if="data"
          v-bind="pageInfo(data.total)"
          @prev="prevPage"
          @next="nextPage"
        />
      </div>
    </div>
  </div>
</template>
