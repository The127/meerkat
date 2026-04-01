<script setup lang="ts">
import { FolderOpen } from 'lucide-vue-next'
import { useProjects } from '@/composables/useProjects'
import MkCardList from '@/components/meerkat/MkCardList.vue'

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
    <div class="mb-6">
      <h1 class="text-xl font-semibold text-foreground mb-1">Projects</h1>
      <p class="text-sm text-muted-foreground">Manage your projects and their SDKs.</p>
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
    </MkCardList>
  </div>
</template>
