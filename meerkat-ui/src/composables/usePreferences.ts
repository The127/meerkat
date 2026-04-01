import { ref, watch, type Ref } from 'vue'

export type ColorScheme = 'system' | 'light' | 'dark'
export type Theme = 'default' | 'blue' | 'green' | 'amber'
export type ColorVision = 'normal' | 'protanopia' | 'deuteranopia' | 'tritanopia'
export type FontSetting = 'system' | 'app' | 'dyslexia'
export type Density = 'comfortable' | 'compact'

export interface Preferences {
  colorScheme: ColorScheme
  theme: Theme
  colorVision: ColorVision
  highContrast: boolean
  font: FontSetting
  density: Density
}

const STORAGE_KEY = 'meerkat-preferences'

const defaults: Preferences = {
  colorScheme: 'system',
  theme: 'default',
  colorVision: 'normal',
  highContrast: false,
  font: 'app',
  density: 'comfortable',
}

function load(): Preferences {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...defaults }
    return { ...defaults, ...JSON.parse(raw) }
  } catch {
    return { ...defaults }
  }
}

function save(prefs: Preferences) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs))
}

// Shared reactive state
const colorScheme: Ref<ColorScheme> = ref(defaults.colorScheme)
const theme: Ref<Theme> = ref(defaults.theme)
const colorVision: Ref<ColorVision> = ref(defaults.colorVision)
const highContrast = ref(false)
const font: Ref<FontSetting> = ref(defaults.font)
const density: Ref<Density> = ref(defaults.density)

let initialized = false

function applyToDOM() {
  const html = document.documentElement

  // Color scheme
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const isDark = colorScheme.value === 'dark' || (colorScheme.value === 'system' && prefersDark)
  html.classList.toggle('dark', isDark)

  // Theme
  html.classList.remove('theme-default', 'theme-blue', 'theme-green', 'theme-amber')
  html.classList.add(`theme-${theme.value}`)

  // Color vision
  if (colorVision.value === 'normal') {
    html.style.filter = ''
  } else {
    html.style.filter = `url(#cv-${colorVision.value})`
  }

  // High contrast
  html.classList.toggle('high-contrast', highContrast.value)

  // Font
  html.classList.toggle('font-system', font.value === 'system')
  html.classList.toggle('font-dyslexia', font.value === 'dyslexia')

  // Density
  html.classList.toggle('density-compact', density.value === 'compact')
}

function persist() {
  save({
    colorScheme: colorScheme.value,
    theme: theme.value,
    colorVision: colorVision.value,
    highContrast: highContrast.value,
    font: font.value,
    density: density.value,
  })
  applyToDOM()
}

export function usePreferences() {
  if (!initialized) {
    const saved = load()
    colorScheme.value = saved.colorScheme
    theme.value = saved.theme
    colorVision.value = saved.colorVision
    highContrast.value = saved.highContrast
    font.value = saved.font
    density.value = saved.density

    // Watch system preference changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyToDOM)

    // Watch all prefs and persist + apply
    watch([colorScheme, theme, colorVision, highContrast, font, density], persist)

    applyToDOM()
    initialized = true
  }

  return {
    colorScheme,
    theme,
    colorVision,
    highContrast,
    font,
    density,
  }
}
