<script setup lang="ts">
import { MkToastContainer } from '@/components/meerkat'
import { usePreferences } from '@/composables/usePreferences'

// Initialize preferences on app mount — applies saved settings to DOM
usePreferences()
</script>

<template>
  <MkToastContainer>
    <RouterView />
  </MkToastContainer>

  <!-- Color vision simulation filters (referenced by usePreferences via CSS filter: url(#cv-xxx)) -->
  <svg class="absolute h-0 w-0" aria-hidden="true">
    <defs>
      <!-- Protanopia (red-blind) -->
      <filter id="cv-protanopia">
        <feColorMatrix type="matrix" values="
          0.567, 0.433, 0,     0, 0
          0.558, 0.442, 0,     0, 0
          0,     0.242, 0.758, 0, 0
          0,     0,     0,     1, 0
        " />
      </filter>
      <!-- Deuteranopia (green-blind) -->
      <filter id="cv-deuteranopia">
        <feColorMatrix type="matrix" values="
          0.625, 0.375, 0,   0, 0
          0.7,   0.3,   0,   0, 0
          0,     0.3,   0.7, 0, 0
          0,     0,     0,   1, 0
        " />
      </filter>
      <!-- Tritanopia (blue-blind) -->
      <filter id="cv-tritanopia">
        <feColorMatrix type="matrix" values="
          0.95, 0.05,  0,     0, 0
          0,    0.433, 0.567, 0, 0
          0,    0.475, 0.525, 0, 0
          0,    0,     0,     1, 0
        " />
      </filter>
    </defs>
  </svg>
</template>
