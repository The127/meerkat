<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card'

defineProps<{
  title?: string
  description?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Card :class="$props.class">
    <CardHeader v-if="title || description || $slots.header">
      <slot name="header">
        <CardTitle v-if="title">{{ title }}</CardTitle>
        <CardDescription v-if="description">{{ description }}</CardDescription>
      </slot>
    </CardHeader>
    <CardContent>
      <slot />
    </CardContent>
    <CardFooter v-if="$slots.footer">
      <slot name="footer" />
    </CardFooter>
  </Card>
</template>
