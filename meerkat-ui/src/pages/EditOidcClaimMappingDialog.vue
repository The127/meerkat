<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkButton, MkInput, MkSpinner } from '@/components/meerkat'
import { useUpdateOidcClaimMapping } from '@/composables/useUpdateOidcClaimMapping'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'
import type { OidcConfigListItem } from '@/lib/types'

const props = defineProps<{ open: boolean; config: OidcConfigListItem | null }>()
const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const toast = useToast()
const { mutateAsync, isPending } = useUpdateOidcClaimMapping()

const subClaim = ref('')
const nameClaim = ref('')
const roleClaim = ref('')
const ownerValues = ref('')
const adminValues = ref('')
const memberValues = ref('')
const errorMessage = ref('')

watch(() => props.config, (c) => {
  if (!c) return
  subClaim.value = c.claim_mapping.sub_claim
  nameClaim.value = c.claim_mapping.name_claim
  roleClaim.value = c.claim_mapping.role_claim
  ownerValues.value = c.claim_mapping.role_values.owner.join(', ')
  adminValues.value = c.claim_mapping.role_values.admin.join(', ')
  memberValues.value = c.claim_mapping.role_values.member.join(', ')
  errorMessage.value = ''
}, { immediate: true })

function close() {
  emit('update:open', false)
}

function splitCsv(value: string): string[] {
  return value.split(',').map(s => s.trim()).filter(s => s.length > 0)
}

async function submit() {
  if (!props.config) return
  errorMessage.value = ''

  try {
    await mutateAsync({
      configId: props.config.id,
      claimMapping: {
        sub_claim: subClaim.value.trim(),
        name_claim: nameClaim.value.trim(),
        role_claim: roleClaim.value.trim(),
        role_values: {
          owner: splitCsv(ownerValues.value),
          admin: splitCsv(adminValues.value),
          member: splitCsv(memberValues.value),
        },
      },
    })
    toast.success('Claim mapping updated')
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
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-lg max-h-[85vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>Edit claim mapping</DialogTitle>
        <DialogDescription>Update how OIDC claims map to roles for <strong>{{ config?.name }}</strong>.</DialogDescription>
      </DialogHeader>

      <form @submit.prevent="submit" class="grid gap-4 py-2">
        <div class="grid gap-1.5">
          <label class="text-xs text-muted-foreground">Name</label>
          <MkInput :model-value="config?.name" disabled />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="grid gap-1.5">
            <label class="text-xs text-muted-foreground">Client ID</label>
            <MkInput :model-value="config?.client_id" disabled />
          </div>
          <div class="grid gap-1.5">
            <label class="text-xs text-muted-foreground">Audience</label>
            <MkInput :model-value="config?.audience" disabled />
          </div>
        </div>

        <div class="grid gap-1.5">
          <label class="text-xs text-muted-foreground">Issuer URL</label>
          <MkInput :model-value="config?.issuer_url" disabled />
        </div>

        <div v-if="config?.discovery_url" class="grid gap-1.5">
          <label class="text-xs text-muted-foreground">Discovery URL</label>
          <MkInput :model-value="config?.discovery_url" disabled />
        </div>

        <div class="border-t border-border pt-4 mt-1">
          <p class="text-sm font-medium text-foreground mb-3">Claim mapping</p>

        <div class="grid grid-cols-3 gap-3">
          <div class="grid gap-1.5">
            <label for="edit-sub" class="text-xs text-muted-foreground">Subject claim</label>
            <MkInput id="edit-sub" v-model="subClaim" :disabled="isPending" />
          </div>
          <div class="grid gap-1.5">
            <label for="edit-name-claim" class="text-xs text-muted-foreground">Name claim</label>
            <MkInput id="edit-name-claim" v-model="nameClaim" :disabled="isPending" />
          </div>
          <div class="grid gap-1.5">
            <label for="edit-role" class="text-xs text-muted-foreground">Role claim</label>
            <MkInput id="edit-role" v-model="roleClaim" :disabled="isPending" />
          </div>
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="grid gap-1.5">
            <label for="edit-owner" class="text-xs text-muted-foreground">Owner values</label>
            <MkInput id="edit-owner" v-model="ownerValues" :disabled="isPending" />
          </div>
          <div class="grid gap-1.5">
            <label for="edit-admin" class="text-xs text-muted-foreground">Admin values</label>
            <MkInput id="edit-admin" v-model="adminValues" :disabled="isPending" />
          </div>
          <div class="grid gap-1.5">
            <label for="edit-member" class="text-xs text-muted-foreground">Member values</label>
            <MkInput id="edit-member" v-model="memberValues" :disabled="isPending" />
          </div>
        </div>
        <p class="text-xs text-muted-foreground">Comma-separated values for each role.</p>
        </div>

        <p v-if="errorMessage" class="text-sm text-destructive">{{ errorMessage }}</p>

        <DialogFooter>
          <MkButton variant="outline" type="button" :disabled="isPending" @click="close">Cancel</MkButton>
          <MkButton type="submit" :disabled="isPending">
            <MkSpinner v-if="isPending" size="sm" class="mr-2" />
            Save
          </MkButton>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
