<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { Button, type ButtonVariants } from '@/components/ui/button'

defineProps<{
  variant?: ButtonVariants['variant']
  size?: ButtonVariants['size']
  as?: string
  class?: HTMLAttributes['class']
  disabled?: boolean
}>()
</script>

<template>
  <Button :variant="variant" :size="size" :as="as" :class="$props.class" :disabled="disabled">
    <slot />
  </Button>
</template>
