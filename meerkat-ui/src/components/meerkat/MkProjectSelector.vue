<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ChevronsUpDown, Check, FolderKanban } from 'lucide-vue-next'
import { useProjects } from '@/composables/useProjects'
import { useCurrentProject } from '@/composables/useCurrentProject'
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuLabel,
} from '@/components/ui/dropdown-menu'

defineProps<{
  collapsed: boolean
}>()

const router = useRouter()
const { data: projectsData } = useProjects()
const { currentProject, slug } = useCurrentProject()

function selectProject(projectSlug: string) {
  router.push({ name: 'project-dashboard', params: { slug: projectSlug } })
}

function viewAllProjects() {
  router.push({ name: 'projects' })
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <button
        :title="currentProject?.name ?? 'Select project'"
        :class="[
          'flex items-center gap-2 rounded-md text-sm hover:bg-accent/50 transition-colors cursor-pointer w-full',
          collapsed ? 'justify-center px-0 py-1.5' : 'px-3 py-1.5',
        ]"
      >
        <div class="w-5 h-5 rounded bg-primary/20 flex items-center justify-center shrink-0 text-primary text-[10px] font-semibold">
          <template v-if="currentProject">{{ currentProject.name.charAt(0).toUpperCase() }}</template>
          <FolderKanban v-else class="w-3 h-3" />
        </div>
        <template v-if="!collapsed">
          <span class="text-sm text-foreground truncate flex-1 text-left">
            {{ currentProject?.name ?? 'Select project' }}
          </span>
          <ChevronsUpDown class="w-4 h-4 text-muted-foreground shrink-0" />
        </template>
      </button>
    </DropdownMenuTrigger>

    <DropdownMenuContent align="start" :side-offset="4" class="w-56">
      <DropdownMenuLabel>Switch project</DropdownMenuLabel>
      <DropdownMenuSeparator />
      <DropdownMenuItem
        v-for="project in projectsData?.items ?? []"
        :key="project.id"
        class="flex items-center justify-between"
        @click="selectProject(project.slug)"
      >
        <span class="truncate">{{ project.name }}</span>
        <Check v-if="project.slug === slug" class="w-4 h-4 shrink-0 text-primary" />
      </DropdownMenuItem>
      <template v-if="!projectsData?.items?.length">
        <div class="px-2 py-1.5 text-sm text-muted-foreground">No projects yet</div>
      </template>
      <DropdownMenuSeparator />
      <DropdownMenuItem @click="viewAllProjects">
        View all projects
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
