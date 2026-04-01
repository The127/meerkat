<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import MkButton from '@/components/meerkat/MkButton.vue'
import MkInput from '@/components/meerkat/MkInput.vue'
import MkSpinner from '@/components/meerkat/MkSpinner.vue'
import { useOrganization } from '@/composables/useOrganization'
import { useCreateProject } from '@/composables/useCreateProject'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'

const router = useRouter()
const toast = useToast()
const { data: org } = useOrganization()
const { mutateAsync, isPending } = useCreateProject()

const name = ref('')
const slug = ref('')
const slugEdited = ref(false)
const errorMessage = ref('')

watch(name, (val) => {
  if (!slugEdited.value) {
    slug.value = val
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9-]+/g, '-')
      .replace(/^-+|-+$/g, '')
  }
})

function onSlugInput(val: string | number) {
  slug.value = String(val)
  slugEdited.value = true
}

function close() {
  router.push({ name: 'projects' })
}

async function submit() {
  if (!name.value.trim() || !slug.value.trim() || !org.value) return

  errorMessage.value = ''

  try {
    await mutateAsync({
      organization_id: org.value.id,
      name: name.value.trim(),
      slug: slug.value.trim(),
    })
    toast.success('Project created')
    close()
  } catch (err) {
    if (err instanceof ApiRequestError) {
      errorMessage.value = err.error.message
    } else {
      errorMessage.value = 'An unexpected error occurred'
    }
  }
}
</script>

<template>
  <Dialog :open="true" @update:open="close">
    <DialogContent @escape-key-down="close" @pointer-down-outside="close">
      <DialogHeader>
        <DialogTitle>Create project</DialogTitle>
        <DialogDescription>Add a new project to start tracking errors.</DialogDescription>
      </DialogHeader>

      <form @submit.prevent="submit" class="grid gap-4 py-2">
        <div class="grid gap-1.5">
          <label for="name" class="text-sm font-medium text-foreground">Name</label>
          <MkInput
            id="name"
            v-model="name"
            placeholder="My Project"
            :disabled="isPending"
          />
        </div>

        <div class="grid gap-1.5">
          <label for="slug" class="text-sm font-medium text-foreground">Slug</label>
          <MkInput
            id="slug"
            :model-value="slug"
            @update:model-value="onSlugInput"
            placeholder="my-project"
            :disabled="isPending"
          />
          <p class="text-xs text-muted-foreground">Lowercase letters, numbers, and hyphens only.</p>
        </div>

        <p v-if="errorMessage" class="text-sm text-destructive">{{ errorMessage }}</p>

        <DialogFooter>
          <MkButton variant="outline" type="button" :disabled="isPending" @click="close">
            Cancel
          </MkButton>
          <MkButton type="submit" :disabled="isPending || !name.trim() || !slug.trim()">
            <MkSpinner v-if="isPending" size="sm" class="mr-2" />
            Create
          </MkButton>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
