<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import { PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent } from 'radix-vue'
import { ChevronsUpDown, Check, FolderKanban, Search } from 'lucide-vue-next'
import { useProjects } from '@/composables/useProjects'
import { useCurrentProject } from '@/composables/useCurrentProject'

defineProps<{
  collapsed: boolean
}>()

const LIMIT = 10

const router = useRouter()
const { currentProject, slug } = useCurrentProject()

const open = ref(false)
const search = ref('')
const debouncedSearch = ref('')
let debounceTimer: ReturnType<typeof setTimeout>
watch(search, (val) => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => { debouncedSearch.value = val }, 250)
})
const searchQuery = computed(() => debouncedSearch.value.trim() || undefined)

const { data: projectsData } = useProjects({ search: searchQuery, limit: ref(LIMIT) })

const projects = computed(() => projectsData.value?.items ?? [])
const total = computed(() => projectsData.value?.total ?? 0)
const hasMore = computed(() => total.value > LIMIT)

watch(open, (isOpen) => {
  if (isOpen) {
    search.value = ''
    nextTick(() => searchInput.value?.focus())
  }
})

const searchInput = ref<HTMLInputElement>()

function selectProject(projectSlug: string) {
  open.value = false
  router.push({ name: 'project-dashboard', params: { slug: projectSlug } })
}

function viewAllProjects() {
  open.value = false
  router.push({ name: 'projects' })
}
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger as-child>
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
    </PopoverTrigger>

    <PopoverPortal>
      <PopoverContent
        side="bottom"
        align="start"
        :side-offset="4"
        class="z-50 w-56 rounded-md border border-border bg-popover p-0 text-popover-foreground shadow-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95"
      >
        <!-- Search -->
        <div class="flex items-center gap-2 px-3 py-2 border-b border-border">
          <Search class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <input
            ref="searchInput"
            v-model="search"
            placeholder="Search projects..."
            class="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
        </div>

        <!-- Project list -->
        <div class="max-h-48 overflow-y-auto p-1">
          <button
            v-for="project in projects"
            :key="project.id"
            class="flex items-center justify-between w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
            @click="selectProject(project.slug)"
          >
            <span class="truncate">{{ project.name }}</span>
            <Check v-if="project.slug === slug" class="w-4 h-4 shrink-0 text-primary" />
          </button>
          <div v-if="hasMore" class="px-2 py-1.5 text-xs text-muted-foreground">
            {{ total - LIMIT }} more — refine your search
          </div>
          <div v-if="!projects.length" class="px-2 py-1.5 text-sm text-muted-foreground">
            No projects found
          </div>
        </div>

        <!-- Footer -->
        <div class="border-t border-border p-1">
          <button
            class="flex items-center w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
            @click="viewAllProjects"
          >
            View all projects
          </button>
        </div>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
