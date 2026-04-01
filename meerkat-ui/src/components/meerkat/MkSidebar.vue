<script setup lang="ts">
import { useRoute } from 'vue-router'
import { Bug, FolderKanban, Settings, Layers, PanelLeftClose, PanelLeftOpen } from 'lucide-vue-next'
import { useSidebar } from '@/composables/useSidebar'

const route = useRoute()
const { collapsed, toggle } = useSidebar()

const navItems = [
  { name: 'Issues', path: '/issues', icon: Bug },
  { name: 'Projects', path: '/projects', icon: FolderKanban },
  { name: 'Settings', path: '/settings', icon: Settings },
]

function isActive(path: string): boolean {
  return route.path === path || route.path.startsWith(path + '/')
}
</script>

<template>
  <aside
    :class="[
      'flex flex-col shrink-0 border-r border-border bg-muted/30 h-screen transition-[width] duration-200',
      collapsed ? 'w-14' : 'w-56',
    ]"
  >
    <!-- Org header — clickable to dashboard -->
    <RouterLink to="/" class="flex items-center gap-2.5 px-3.5 h-14 border-b border-border hover:bg-accent/30 transition-colors">
      <div class="w-7 h-7 rounded bg-primary/90 flex items-center justify-center shrink-0">
        <Layers class="w-4 h-4 text-white" />
      </div>
      <span v-if="!collapsed" class="text-sm font-semibold text-foreground truncate">Meerkat</span>
    </RouterLink>

    <!-- Nav -->
    <nav class="flex-1 px-2 py-3 space-y-0.5">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        :title="collapsed ? item.name : undefined"
        :class="[
          'flex items-center gap-2.5 rounded-md text-sm transition-colors',
          collapsed ? 'justify-center px-0 py-1.5' : 'px-3 py-1.5',
          isActive(item.path)
            ? 'bg-accent text-accent-foreground font-medium'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
        ]"
      >
        <component :is="item.icon" class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">{{ item.name }}</span>
      </RouterLink>
    </nav>

    <!-- Collapse toggle -->
    <div class="px-2 py-3 border-t border-border">
      <button
        :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        :class="[
          'flex items-center gap-2.5 rounded-md text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors cursor-pointer w-full',
          collapsed ? 'justify-center px-0 py-1.5' : 'px-3 py-1.5',
        ]"
        @click="toggle"
      >
        <PanelLeftClose v-if="!collapsed" class="w-4 h-4 shrink-0" />
        <PanelLeftOpen v-else class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">Collapse</span>
      </button>
    </div>
  </aside>
</template>
