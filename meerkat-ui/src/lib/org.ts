/**
 * Extracts the org slug from the current hostname.
 *
 * - `acme.meerkat.dev`  → `"acme"`
 * - `meerkat.dev`       → `null` (master org)
 * - `localhost`         → `null` (dev, master org)
 */
export function resolveOrg(host: string, baseDomain: string): string | null {
  // Strip port
  const hostname = host.replace(/:\d+$/, '')

  // localhost / IP → master org
  if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname.startsWith('[')) {
    return null
  }

  if (!hostname.endsWith(baseDomain)) {
    return null
  }

  const prefix = hostname.slice(0, -(baseDomain.length + 1)) // +1 for the dot
  return prefix.length > 0 ? prefix : null
}
