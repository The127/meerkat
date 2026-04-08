<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { AlertCircle, KeyRound, Layers, PanelLeftClose, PanelLeftOpen, Settings, Shield, UserCog, Users, Wrench } from 'lucide-vue-next'
import { useSidebar } from '@/composables/useSidebar'
import { useCurrentUser } from '@/composables/useCurrentUser'
import MkProjectSelector from './MkProjectSelector.vue'

const route = useRoute()
const { collapsed, toggle } = useSidebar()
const { canManageMembers } = useCurrentUser()

const slug = computed(() => {
  const param = route.params.slug
  return typeof param === 'string' ? param : undefined
})

const navItems = computed(() => {
  if (!slug.value) return []
  return [
    {
      name: 'Issues',
      path: `/projects/${slug.value}/issues`,
      icon: AlertCircle,
    },
    {
      name: 'Client Keys',
      path: `/projects/${slug.value}/keys`,
      icon: KeyRound,
    },
    {
      name: 'Roles',
      path: `/projects/${slug.value}/roles`,
      icon: Shield,
    },
    {
      name: 'Members',
      path: `/projects/${slug.value}/members`,
      icon: UserCog,
    },
    {
      name: 'Settings',
      path: `/projects/${slug.value}/settings`,
      icon: Wrench,
    },
  ]
})

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
    <!-- Org header -->
    <RouterLink to="/" class="flex items-center gap-2.5 px-3.5 h-14 border-b border-border hover:bg-accent/30 transition-colors">
      <div class="w-7 h-7 rounded bg-primary/90 flex items-center justify-center shrink-0">
        <Layers class="w-4 h-4 text-white" />
      </div>
      <span v-if="!collapsed" class="text-sm font-semibold text-foreground truncate">Meerkat</span>
    </RouterLink>

    <!-- Project selector -->
    <div class="px-2 pt-3 pb-2">
      <MkProjectSelector :collapsed="collapsed" />
    </div>

    <div class="mx-2 border-t border-border" />

    <!-- Nav -->
    <nav class="flex-1 px-2 py-2 space-y-0.5">
      <RouterLink
        v-for="item in navItems"
        :key="item.name"
        :to="item.path"
        :title="collapsed ? item.name : undefined"
        :class="[
          'flex items-center gap-2.5 rounded-md text-sm transition-colors px-3 py-1.5',
          isActive(item.path)
            ? 'bg-accent text-accent-foreground font-medium'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
        ]"
      >
        <component :is="item.icon" class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">{{ item.name }}</span>
      </RouterLink>
    </nav>

    <!-- Members + Settings -->
    <div class="px-2 py-2 border-t border-border space-y-0.5">
      <RouterLink
        v-if="canManageMembers"
        to="/members"
        :title="collapsed ? 'Members' : undefined"
        :class="[
          'flex items-center gap-2.5 rounded-md text-sm transition-colors px-3 py-1.5',
          isActive('/members')
            ? 'bg-accent text-accent-foreground font-medium'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
        ]"
      >
        <Users class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">Members</span>
      </RouterLink>
      <RouterLink
        to="/settings"
        :title="collapsed ? 'Settings' : undefined"
        :class="[
          'flex items-center gap-2.5 rounded-md text-sm transition-colors px-3 py-1.5',
          isActive('/settings')
            ? 'bg-accent text-accent-foreground font-medium'
            : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
        ]"
      >
        <Settings class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">Settings</span>
      </RouterLink>
    </div>

    <!-- Collapse toggle -->
    <div class="px-2 py-3 border-t border-border">
      <button
        :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        class="flex items-center gap-2.5 rounded-md text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors cursor-pointer w-full px-3 py-1.5"
        @click="toggle"
      >
        <PanelLeftClose v-if="!collapsed" class="w-4 h-4 shrink-0" />
        <PanelLeftOpen v-else class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">Collapse</span>
      </button>
    </div>
  </aside>
</template>
