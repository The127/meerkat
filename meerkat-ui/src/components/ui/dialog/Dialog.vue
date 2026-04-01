<script setup lang="ts">
import { DialogRoot, type DialogRootProps, useForwardProps } from 'radix-vue'

const props = defineProps<DialogRootProps>()
const forwarded = useForwardProps(props)
</script>

<template>
  <DialogRoot v-bind="forwarded">
    <slot />
  </DialogRoot>
</template>
