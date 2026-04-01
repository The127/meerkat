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
