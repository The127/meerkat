/** Matches backend OidcConfigDto */
export interface OidcConfig {
  name: string
  client_id: string
  issuer_url: string
  audience: string
}

export interface ApiError {
  code: string
  message: string
}
