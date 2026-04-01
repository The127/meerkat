<script setup lang="ts">
import { computed } from 'vue'
import { MkButton, MkSpinner } from '@/components/meerkat'
import { useOidcConfig } from '@/composables/useOidcConfig'
import { useAuth } from '@/composables/useAuth'

const { data: oidcConfig, isLoading, isError } = useOidcConfig()
const { login } = useAuth()

const providerName = computed(() => oidcConfig.value?.name ?? 'SSO')

async function handleLogin() {
  await login()
}
</script>

<template>
  <div class="flex min-h-screen">
    <!-- Left panel — dark, techy -->
    <div class="hidden lg:flex lg:w-1/2 relative bg-[#0c0a13] overflow-hidden">
      <!-- Grid pattern -->
      <div class="absolute inset-0 opacity-[0.15]"
        :style="{
          backgroundImage: 'linear-gradient(rgba(108,95,199,0.6) 1px, transparent 1px), linear-gradient(90deg, rgba(108,95,199,0.6) 1px, transparent 1px)',
          backgroundSize: '48px 48px',
        }"
      />

      <!-- Gradient glow -->
      <div class="absolute -bottom-24 -left-24 w-[28rem] h-[28rem] bg-primary/40 rounded-full blur-[128px]" />
      <div class="absolute -top-12 -right-12 w-80 h-80 bg-primary/25 rounded-full blur-[96px]" />
      <div class="absolute top-1/2 left-1/3 w-48 h-48 bg-primary/15 rounded-full blur-[80px]" />

      <!-- Accent lines -->
      <div class="absolute top-0 left-32 w-px h-full bg-gradient-to-b from-transparent via-primary/50 to-transparent" />
      <div class="absolute top-0 left-[calc(50%+2rem)] w-px h-full bg-gradient-to-b from-transparent via-primary/35 to-transparent" />
      <div class="absolute top-1/3 left-0 w-full h-px bg-gradient-to-r from-transparent via-primary/35 to-transparent" />
      <div class="absolute top-2/3 left-0 w-full h-px bg-gradient-to-r from-primary/20 via-primary/10 to-transparent" />

      <!-- Content -->
      <div class="relative z-10 flex flex-col justify-between p-16 w-full">
        <div>
          <!-- Logo -->
          <div class="flex items-center gap-3 mb-4">
            <div class="w-8 h-8 rounded bg-primary/90 flex items-center justify-center">
              <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2L2 7l10 5 10-5-10-5z" />
                <path d="M2 17l10 5 10-5" />
                <path d="M2 12l10 5 10-5" />
              </svg>
            </div>
            <span class="text-white/90 text-xl font-semibold tracking-tight">meerkat</span>
          </div>
        </div>

        <div class="space-y-6">
          <h1 class="text-5xl font-semibold text-white leading-tight">
            Error tracking,<br />
            <span class="text-primary">open source.</span>
          </h1>
          <p class="text-white/50 text-lg max-w-md leading-relaxed">
            Lightweight error monitoring you can self-host. Capture, triage, and resolve. No vendor lock-in.
          </p>

          <!-- Decorative stats -->
          <div class="flex gap-10 pt-6">
            <div>
              <div class="text-3xl font-semibold text-white font-mono">99.9%</div>
              <div class="text-xs text-white/40 mt-1 uppercase tracking-wider">Uptime</div>
            </div>
            <div class="w-px bg-white/15" />
            <div>
              <div class="text-3xl font-semibold text-white font-mono">&lt;50ms</div>
              <div class="text-xs text-white/40 mt-1 uppercase tracking-wider">Ingestion</div>
            </div>
            <div class="w-px bg-white/15" />
            <div>
              <div class="text-3xl font-semibold text-white font-mono">OSS</div>
              <div class="text-xs text-white/40 mt-1 uppercase tracking-wider">Licensed</div>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-4 text-white/20 text-xs">
          <span>&copy; {{ new Date().getFullYear() }} Meerkat</span>
          <a href="https://github.com/The127/meerkat" target="_blank" rel="noopener" class="inline-flex items-center gap-1.5 text-white/30 hover:text-white/60 transition-colors">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>
            GitHub
          </a>
        </div>
      </div>
    </div>

    <!-- Right panel — light, login form -->
    <div class="flex-1 flex items-center justify-center bg-background p-8">
      <div class="w-full max-w-sm">
        <!-- Mobile logo -->
        <div class="flex items-center gap-3 mb-10 lg:hidden">
          <div class="w-8 h-8 rounded bg-primary/90 flex items-center justify-center">
            <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
          </div>
          <span class="text-foreground text-xl font-semibold tracking-tight">meerkat</span>
        </div>

        <!-- Login card -->
        <div class="rounded-lg border border-border bg-card p-8 shadow-sm">
          <div class="space-y-2 mb-8">
            <h2 class="text-xl font-semibold text-foreground">Sign in</h2>
            <p class="text-sm text-muted-foreground">
              Authenticate with your identity provider to continue.
            </p>
          </div>

          <!-- Loading state -->
          <div v-if="isLoading" class="flex flex-col items-center gap-3 py-8">
            <MkSpinner size="md" />
            <p class="text-sm text-muted-foreground">Loading configuration...</p>
          </div>

          <!-- Error state -->
          <div v-else-if="isError" class="rounded-md border border-destructive/30 bg-destructive/5 p-4">
            <p class="text-sm text-destructive font-medium">Unable to load authentication configuration.</p>
            <p class="text-xs text-destructive/70 mt-1">Check that the backend is running and the organization exists.</p>
          </div>

          <!-- Ready state -->
          <div v-else class="space-y-4">
            <MkButton class="w-full" size="lg" @click="handleLogin">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
              </svg>
              Continue with {{ providerName }}
            </MkButton>
            <p class="text-center text-xs text-muted-foreground">
              You'll be redirected to your identity provider.
            </p>
          </div>
        </div>

        <p class="text-center text-xs text-muted-foreground mt-6">
          Secured with OpenID Connect
        </p>
      </div>
    </div>
  </div>
</template>
