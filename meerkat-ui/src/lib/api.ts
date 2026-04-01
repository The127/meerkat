import type { ApiError } from './types'

export class ApiRequestError extends Error {
  status: number
  error: ApiError

  constructor(status: number, error: ApiError) {
    super(error.message)
    this.name = 'ApiRequestError'
    this.status = status
    this.error = error
  }
}

let tokenProvider: (() => Promise<string | null>) | null = null

export function setTokenProvider(provider: () => Promise<string | null>) {
  tokenProvider = provider
}

export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers)

  if (tokenProvider) {
    const token = await tokenProvider()
    if (token) {
      headers.set('Authorization', `Bearer ${token}`)
    }
  }

  if (!headers.has('Content-Type') && options.body) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(path, { ...options, headers })

  if (!response.ok) {
    const error: ApiError = await response.json().catch(() => ({
      code: 'unknown',
      message: response.statusText,
    }))
    throw new ApiRequestError(response.status, error)
  }

  if (response.status === 204) {
    return undefined as T
  }

  return response.json()
}
