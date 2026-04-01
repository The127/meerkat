<script setup lang="ts">
import { usePreferences, type ColorScheme, type Theme, type ColorVision, type FontSetting, type Density } from '@/composables/usePreferences'
import { MkCard, MkButton, MkBadge, MkAlert } from '@/components/meerkat'
import { Bug, FolderKanban, Settings, Layers, CircleUser } from 'lucide-vue-next'

const { colorScheme, theme, colorVision, highContrast, font, density } = usePreferences()

const colorSchemeOptions: { value: ColorScheme; label: string }[] = [
  { value: 'system', label: 'System' },
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
]

const themeOptions: { value: Theme; label: string; swatch: string }[] = [
  { value: 'default', label: 'Purple', swatch: '#6c5fc7' },
  { value: 'blue', label: 'Blue', swatch: '#3b82f6' },
  { value: 'green', label: 'Green', swatch: '#16a34a' },
  { value: 'amber', label: 'Amber', swatch: '#d97706' },
]

const colorVisionOptions: { value: ColorVision; label: string }[] = [
  { value: 'normal', label: 'Normal vision' },
  { value: 'protanopia', label: 'Protanopia (red-blind)' },
  { value: 'deuteranopia', label: 'Deuteranopia (green-blind)' },
  { value: 'tritanopia', label: 'Tritanopia (blue-blind)' },
]

const fontOptions: { value: FontSetting; label: string }[] = [
  { value: 'app', label: 'App default (Rubik)' },
  { value: 'system', label: 'System font' },
  { value: 'dyslexia', label: 'Dyslexia-friendly (OpenDyslexic)' },
]

const densityOptions: { value: Density; label: string }[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' },
]
</script>

<template>
  <div>
    <h1 class="text-xl font-semibold text-foreground mb-1">Profile</h1>
    <p class="text-sm text-muted-foreground mb-6">Manage your appearance and accessibility preferences.</p>

    <div class="grid grid-cols-1 lg:grid-cols-[1fr_22rem] gap-6 items-start">
      <!-- Settings column -->
      <div class="space-y-6">
        <!-- Appearance -->
        <MkCard title="Appearance">
          <div class="space-y-5">
            <!-- Color scheme -->
            <div>
              <label class="text-sm font-medium text-foreground block mb-2">Color scheme</label>
              <div class="flex gap-2">
                <button
                  v-for="opt in colorSchemeOptions"
                  :key="opt.value"
                  :class="[
                    'px-3 py-1.5 rounded-md text-sm border transition-colors cursor-pointer',
                    colorScheme === opt.value
                      ? 'border-primary bg-primary/10 text-primary font-medium'
                      : 'border-border text-muted-foreground hover:border-primary/50',
                  ]"
                  @click="colorScheme = opt.value"
                >
                  {{ opt.label }}
                </button>
              </div>
            </div>

            <!-- Theme -->
            <div>
              <label class="text-sm font-medium text-foreground block mb-2">Theme color</label>
              <div class="flex gap-2">
                <button
                  v-for="opt in themeOptions"
                  :key="opt.value"
                  :title="opt.label"
                  :class="[
                    'w-8 h-8 rounded-full border-2 transition-all cursor-pointer',
                    theme === opt.value
                      ? 'border-foreground scale-110'
                      : 'border-transparent hover:scale-105',
                  ]"
                  :style="{ backgroundColor: opt.swatch }"
                  @click="theme = opt.value"
                />
              </div>
            </div>

            <!-- Density -->
            <div>
              <label class="text-sm font-medium text-foreground block mb-2">Density</label>
              <div class="flex gap-2">
                <button
                  v-for="opt in densityOptions"
                  :key="opt.value"
                  :class="[
                    'px-3 py-1.5 rounded-md text-sm border transition-colors cursor-pointer',
                    density === opt.value
                      ? 'border-primary bg-primary/10 text-primary font-medium'
                      : 'border-border text-muted-foreground hover:border-primary/50',
                  ]"
                  @click="density = opt.value"
                >
                  {{ opt.label }}
                </button>
              </div>
            </div>
          </div>
        </MkCard>

        <!-- Accessibility -->
        <MkCard title="Accessibility">
          <div class="space-y-5">
            <!-- Color vision -->
            <div>
              <label class="text-sm font-medium text-foreground block mb-2">Color vision</label>
              <select
                :value="colorVision"
                class="w-full rounded-md border border-input bg-transparent px-3 py-1.5 text-sm text-foreground cursor-pointer"
                @change="colorVision = ($event.target as HTMLSelectElement).value as ColorVision"
              >
                <option v-for="opt in colorVisionOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>

            <!-- High contrast -->
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium text-foreground">High contrast</p>
                <p class="text-xs text-muted-foreground">Increase border and text contrast</p>
              </div>
              <button
                :class="[
                  'relative inline-flex h-6 w-11 items-center rounded-full transition-colors cursor-pointer',
                  highContrast ? 'bg-primary' : 'bg-border',
                ]"
                role="switch"
                :aria-checked="highContrast"
                @click="highContrast = !highContrast"
              >
                <span
                  :class="[
                    'inline-block h-4 w-4 rounded-full bg-white transition-transform',
                    highContrast ? 'translate-x-6' : 'translate-x-1',
                  ]"
                />
              </button>
            </div>
          </div>
        </MkCard>

        <!-- Typography -->
        <MkCard title="Typography">
          <div>
            <label class="text-sm font-medium text-foreground block mb-2">Font</label>
            <div class="flex flex-col gap-2">
              <button
                v-for="opt in fontOptions"
                :key="opt.value"
                :class="[
                  'px-3 py-2 rounded-md text-sm border text-left transition-colors cursor-pointer',
                  font === opt.value
                    ? 'border-primary bg-primary/10 text-primary font-medium'
                    : 'border-border text-muted-foreground hover:border-primary/50',
                ]"
                @click="font = opt.value"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>
        </MkCard>
      </div>

      <!-- Preview column -->
      <div class="hidden lg:block sticky top-6">
        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-3">Live preview</p>
        <div class="rounded-lg border border-border overflow-hidden shadow-sm bg-background text-foreground">
          <!-- Mini topbar -->
          <div class="flex items-center justify-between h-9 px-3 border-b border-border bg-background">
            <div class="flex items-center gap-1.5">
              <div class="w-5 h-5 rounded bg-primary/90 flex items-center justify-center">
                <Layers class="w-3 h-3 text-white" />
              </div>
              <span class="text-xs font-semibold">Meerkat</span>
            </div>
            <div class="flex items-center gap-1 text-muted-foreground">
              <CircleUser class="w-3 h-3" />
              <span class="text-[10px]">User</span>
            </div>
          </div>

          <div class="flex">
            <!-- Mini sidebar -->
            <div class="w-10 border-r border-border bg-muted/30 py-2 space-y-1">
              <div class="flex justify-center">
                <Bug class="w-3 h-3 text-primary" />
              </div>
              <div class="flex justify-center">
                <FolderKanban class="w-3 h-3 text-muted-foreground" />
              </div>
              <div class="flex justify-center">
                <Settings class="w-3 h-3 text-muted-foreground" />
              </div>
            </div>

            <!-- Mini content -->
            <div class="flex-1 p-3 space-y-2.5">
              <!-- Sample card -->
              <div class="rounded-md border border-border bg-card p-2.5">
                <p class="text-xs font-medium text-card-foreground">TypeError in auth.ts</p>
                <p class="text-[10px] text-muted-foreground mt-0.5">Cannot read property 'token' of undefined</p>
                <div class="flex items-center gap-1.5 mt-2">
                  <MkBadge variant="error" class="text-[9px] px-1 py-0">Critical</MkBadge>
                  <MkBadge variant="warning" class="text-[9px] px-1 py-0">1.2k</MkBadge>
                </div>
              </div>

              <!-- Sample alert -->
              <MkAlert variant="success" title="Resolved" class="text-[10px] py-1.5 px-2.5 [&_h5]:text-[10px] [&_h5]:mb-0">
                <span class="text-[10px]">3 issues resolved in the last hour.</span>
              </MkAlert>

              <!-- Sample buttons -->
              <div class="flex gap-1.5">
                <MkButton size="sm" class="text-[10px] h-6 px-2">Primary</MkButton>
                <MkButton size="sm" variant="outline" class="text-[10px] h-6 px-2">Outline</MkButton>
                <MkButton size="sm" variant="secondary" class="text-[10px] h-6 px-2">Secondary</MkButton>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
