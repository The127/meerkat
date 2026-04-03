<script setup lang="ts">
import { ref } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { MkButton, MkInput, MkSpinner } from '@/components/meerkat'
import { useAddOidcConfig } from '@/composables/useAddOidcConfig'
import { useToast } from '@/composables/useToast'
import { ApiRequestError } from '@/lib/api'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const toast = useToast()
const { mutateAsync, isPending } = useAddOidcConfig()

const name = ref('')
const clientId = ref('')
const issuerUrl = ref('')
const audience = ref('')
const discoveryUrl = ref('')
const subClaim = ref('sub')
const nameClaim = ref('preferred_username')
const roleClaim = ref('roles')
const ownerValues = ref('owner')
const adminValues = ref('admin')
const memberValues = ref('member')
const errorMessage = ref('')

function close() {
  emit('update:open', false)
}

function resetForm() {
  name.value = ''
  clientId.value = ''
  issuerUrl.value = ''
  audience.value = ''
  discoveryUrl.value = ''
  subClaim.value = 'sub'
  nameClaim.value = 'preferred_username'
  roleClaim.value = 'roles'
  ownerValues.value = 'owner'
  adminValues.value = 'admin'
  memberValues.value = 'member'
  errorMessage.value = ''
}

function splitCsv(value: string): string[] {
  return value.split(',').map(s => s.trim()).filter(s => s.length > 0)
}

async function submit() {
  if (!name.value.trim() || !clientId.value.trim() || !issuerUrl.value.trim() || !audience.value.trim()) return
  errorMessage.value = ''

  try {
    await mutateAsync({
      name: name.value.trim(),
      client_id: clientId.value.trim(),
      issuer_url: issuerUrl.value.trim(),
      audience: audience.value.trim(),
      discovery_url: discoveryUrl.value.trim() || undefined,
      sub_claim: subClaim.value.trim(),
      name_claim: nameClaim.value.trim(),
      role_claim: roleClaim.value.trim(),
      owner_values: splitCsv(ownerValues.value),
      admin_values: splitCsv(adminValues.value),
      member_values: splitCsv(memberValues.value),
    })
    toast.success('OIDC configuration added')
    resetForm()
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
        <DialogTitle>Add OIDC configuration</DialogTitle>
        <DialogDescription>Add a new identity provider. It will be created as a draft.</DialogDescription>
      </DialogHeader>

      <form @submit.prevent="submit" class="grid gap-4 py-2">
        <div class="grid gap-1.5">
          <label for="oidc-name" class="text-sm font-medium text-foreground">Name</label>
          <MkInput id="oidc-name" v-model="name" placeholder="Company SSO" :disabled="isPending" />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="grid gap-1.5">
            <label for="oidc-client-id" class="text-sm font-medium text-foreground">Client ID</label>
            <MkInput id="oidc-client-id" v-model="clientId" placeholder="my-app" :disabled="isPending" />
          </div>
          <div class="grid gap-1.5">
            <label for="oidc-audience" class="text-sm font-medium text-foreground">Audience</label>
            <MkInput id="oidc-audience" v-model="audience" placeholder="my-api" :disabled="isPending" />
          </div>
        </div>

        <div class="grid gap-1.5">
          <label for="oidc-issuer" class="text-sm font-medium text-foreground">Issuer URL</label>
          <MkInput id="oidc-issuer" v-model="issuerUrl" placeholder="https://auth.example.com" :disabled="isPending" />
        </div>

        <div class="grid gap-1.5">
          <label for="oidc-discovery" class="text-sm font-medium text-foreground">Discovery URL <span class="text-muted-foreground font-normal">(optional)</span></label>
          <MkInput id="oidc-discovery" v-model="discoveryUrl" placeholder="https://auth.example.com/.well-known/openid-configuration" :disabled="isPending" />
        </div>

        <div class="border-t border-border pt-4 mt-1">
          <p class="text-sm font-medium text-foreground mb-3">Claim mapping</p>

          <div class="grid grid-cols-3 gap-3">
            <div class="grid gap-1.5">
              <label for="oidc-sub" class="text-xs text-muted-foreground">Subject claim</label>
              <MkInput id="oidc-sub" v-model="subClaim" :disabled="isPending" />
            </div>
            <div class="grid gap-1.5">
              <label for="oidc-name-claim" class="text-xs text-muted-foreground">Name claim</label>
              <MkInput id="oidc-name-claim" v-model="nameClaim" :disabled="isPending" />
            </div>
            <div class="grid gap-1.5">
              <label for="oidc-role" class="text-xs text-muted-foreground">Role claim</label>
              <MkInput id="oidc-role" v-model="roleClaim" :disabled="isPending" />
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3 mt-3">
            <div class="grid gap-1.5">
              <label for="oidc-owner" class="text-xs text-muted-foreground">Owner values</label>
              <MkInput id="oidc-owner" v-model="ownerValues" :disabled="isPending" />
            </div>
            <div class="grid gap-1.5">
              <label for="oidc-admin" class="text-xs text-muted-foreground">Admin values</label>
              <MkInput id="oidc-admin" v-model="adminValues" :disabled="isPending" />
            </div>
            <div class="grid gap-1.5">
              <label for="oidc-member" class="text-xs text-muted-foreground">Member values</label>
              <MkInput id="oidc-member" v-model="memberValues" :disabled="isPending" />
            </div>
          </div>
          <p class="text-xs text-muted-foreground mt-1.5">Comma-separated values for each role.</p>
        </div>

        <p v-if="errorMessage" class="text-sm text-destructive">{{ errorMessage }}</p>

        <DialogFooter>
          <MkButton variant="outline" type="button" :disabled="isPending" @click="close">Cancel</MkButton>
          <MkButton type="submit" :disabled="isPending || !name.trim() || !clientId.trim() || !issuerUrl.trim() || !audience.trim()">
            <MkSpinner v-if="isPending" size="sm" class="mr-2" />
            Add
          </MkButton>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
