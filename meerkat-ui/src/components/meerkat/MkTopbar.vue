<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { LogOut, Settings, ChevronDown, CircleUser } from 'lucide-vue-next'
import { useAuth } from '@/composables/useAuth'
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuLabel,
} from '@/components/ui/dropdown-menu'

const router = useRouter()
const { user, logout } = useAuth()

const displayName = computed(() => {
  const profile = user.value?.profile
  if (!profile) return 'User'
  return profile.name || profile.preferred_username || profile.email || 'User'
})

const email = computed(() => user.value?.profile?.email ?? '')

async function handleLogout() {
  try {
    await logout()
  } catch {
    router.push('/auth/login')
  }
}
</script>

<template>
  <header class="flex items-center justify-end h-14 px-6 border-b border-border bg-background">
    <DropdownMenu>
      <DropdownMenuTrigger>
        <button class="flex items-center gap-2 px-2.5 py-1.5 rounded-md text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer">
          <CircleUser class="w-4 h-4" />
          <span>{{ displayName }}</span>
          <ChevronDown class="w-3.5 h-3.5 opacity-50" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent class="w-52">
        <DropdownMenuLabel class="font-normal">
          <div class="flex flex-col gap-0.5">
            <p class="text-sm font-medium">{{ displayName }}</p>
            <p v-if="email" class="text-xs text-muted-foreground truncate">{{ email }}</p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem @click="router.push('/profile')">
          <Settings class="w-4 h-4" />
          Profile
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem @click="handleLogout">
          <LogOut class="w-4 h-4" />
          Sign out
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </header>
</template>
