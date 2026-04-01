<script setup lang="ts">
import { FolderOpen, Plus } from 'lucide-vue-next'
import { RouterLink, RouterView } from 'vue-router'
import { useProjects } from '@/composables/useProjects'
import MkCardList from '@/components/meerkat/MkCardList.vue'
import MkButton from '@/components/meerkat/MkButton.vue'

const { data, isLoading } = useProjects()

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
      <RouterLink v-if="data?.items?.length" :to="{ name: 'projects-new' }">
        <MkButton size="sm">
          <Plus class="h-4 w-4 mr-1.5" />
          Create Project
        </MkButton>
      </RouterLink>
    </div>

    <MkCardList
      :items="data?.items ?? []"
      :loading="isLoading"
      :empty-icon="FolderOpen"
      empty-title="No projects yet"
      empty-description="Create your first project to start tracking errors."
    >
      <template #item="{ item }">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-foreground">{{ item.name }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">{{ item.slug }}</p>
          </div>
          <span class="text-xs text-muted-foreground">{{ formatDate(item.created_at) }}</span>
        </div>
      </template>
      <template #empty>
        <RouterLink :to="{ name: 'projects-new' }">
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
