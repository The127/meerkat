<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { Alert, AlertTitle, AlertDescription, type AlertVariants } from '@/components/ui/alert'

defineProps<{
  variant?: AlertVariants['variant']
  title?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Alert :variant="variant" :class="$props.class">
    <AlertTitle v-if="title">{{ title }}</AlertTitle>
    <AlertDescription>
      <slot />
    </AlertDescription>
  </Alert>
</template>
