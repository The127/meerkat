<script setup lang="ts">
import { ref, nextTick, type HTMLAttributes } from 'vue'
import { Pencil } from 'lucide-vue-next'

const props = defineProps<{
  modelValue: string
  disabled?: boolean
  class?: HTMLAttributes['class']
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'save': [value: string]
}>()

const editing = ref(false)
const draft = ref('')
const inputRef = ref<HTMLInputElement>()

async function startEditing() {
  if (props.disabled) return
  draft.value = props.modelValue
  editing.value = true
  await nextTick()
  inputRef.value?.focus()
  inputRef.value?.select()
}

function save() {
  editing.value = false
  const trimmed = draft.value.trim()
  if (!trimmed || trimmed === props.modelValue) return
  emit('update:modelValue', trimmed)
  emit('save', trimmed)
}

function cancel() {
  editing.value = false
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    save()
  } else if (e.key === 'Escape') {
    cancel()
  }
}
</script>

<template>
  <input
    v-if="editing"
    ref="inputRef"
    v-model="draft"
    :class="[
      'bg-transparent border-b-2 border-primary outline-none',
      $props.class,
    ]"
    @blur="save"
    @keydown="onKeydown"
  />
  <button
    v-else
    type="button"
    :class="[
      'group inline-flex items-center gap-2 text-left',
      disabled ? 'cursor-default' : 'cursor-pointer',
      $props.class,
    ]"
    :disabled="disabled"
    @click="startEditing"
  >
    <span>{{ modelValue }}</span>
    <Pencil
      v-if="!disabled"
      class="w-3.5 h-3.5 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity"
    />
  </button>
</template>
