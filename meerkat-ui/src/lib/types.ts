/** Matches backend OidcConfigDto */
export interface OidcConfig {
  name: string
  client_id: string
  issuer_url: string
  audience: string
  discovery_url?: string
}

export interface ApiError {
  code: string
  message: string
}

export interface Organization {
  id: string
  slug: string
  name: string
}

export interface Project {
  id: string
  organization_id: string
  name: string
  slug: string
  created_at: string
  updated_at: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
}

export interface ClaimMapping {
  sub_claim: string
  name_claim: string
  role_claim: string
  owner_values: string[]
  admin_values: string[]
  member_values: string[]
}

export interface OidcConfigListItem {
  id: string
  name: string
  client_id: string
  issuer_url: string
  audience: string
  discovery_url?: string
  claim_mapping: ClaimMapping
  status: 'draft' | 'active' | 'inactive'
}

export interface CurrentUser {
  member_id: string
  preferred_name: string
  org_permissions: string[]
  project_permissions: Record<string, string[]>
}
