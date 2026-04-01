<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { Input } from '@/components/ui/input'

defineProps<{
  class?: HTMLAttributes['class']
  modelValue?: string | number
  placeholder?: string
  type?: string
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string | number]
}>()
</script>

<template>
  <Input
    :class="$props.class"
    :model-value="modelValue"
    :placeholder="placeholder"
    :type="type"
    :disabled="disabled"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>
