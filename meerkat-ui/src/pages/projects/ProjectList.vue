<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { FolderOpen, Plus, Search } from 'lucide-vue-next'
import { RouterLink, RouterView } from 'vue-router'
import { useProjects } from '@/composables/useProjects'
import { useCurrentUser } from '@/composables/useCurrentUser'
import MkCardList from '@/components/meerkat/MkCardList.vue'
import MkButton from '@/components/meerkat/MkButton.vue'
import MkInput from '@/components/meerkat/MkInput.vue'

const search = ref('')
const debouncedSearch = ref('')
let debounceTimer: ReturnType<typeof setTimeout>
watch(search, (val) => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => { debouncedSearch.value = val }, 250)
})
const searchQuery = computed(() => debouncedSearch.value.trim() || undefined)

const { data, isLoading } = useProjects({ search: searchQuery })
const { canCreateProject } = useCurrentUser()

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-xl font-semibold text-foreground mb-1">Projects</h1>
        <p class="text-sm text-muted-foreground">Manage your projects and their SDKs.</p>
      </div>
      <RouterLink v-if="canCreateProject && (data?.items?.length || searchQuery)" :to="{ name: 'projects-new' }">
        <MkButton size="sm">
          <Plus class="h-4 w-4 mr-1.5" />
          Create Project
        </MkButton>
      </RouterLink>
    </div>

    <div v-if="data?.total || searchQuery" class="relative mb-4 max-w-xs">
      <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground pointer-events-none" />
      <MkInput
        v-model="search"
        placeholder="Search projects..."
        class="pl-8 h-8 text-sm"
      />
    </div>

    <MkCardList
      :items="data?.items ?? []"
      :loading="isLoading"
      :empty-icon="FolderOpen"
      :empty-title="searchQuery ? 'No projects found' : 'No projects yet'"
      :empty-description="searchQuery ? 'Try a different search term.' : 'Create your first project to start tracking errors.'"
    >
      <template #item="{ item }">
        <RouterLink
          :to="{ name: 'project-dashboard', params: { slug: item.slug } }"
          class="flex items-center justify-between -mx-4 -my-3 px-4 py-3"
        >
          <div>
            <p class="text-sm font-medium text-foreground group-hover:text-primary">{{ item.name }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">{{ item.slug }}</p>
          </div>
          <span class="text-xs text-muted-foreground">{{ formatDate(item.created_at) }}</span>
        </RouterLink>
      </template>
      <template #empty>
        <RouterLink v-if="canCreateProject && !searchQuery" :to="{ name: 'projects-new' }">
          <MkButton size="sm">
            <Plus class="h-4 w-4 mr-1.5" />
            Create Project
          </MkButton>
        </RouterLink>
      </template>
    </MkCardList>

    <RouterView />
  </div>
</template>
