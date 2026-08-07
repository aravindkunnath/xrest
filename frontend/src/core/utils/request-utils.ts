/**
 * Request Utilities
 *
 * Pure utility functions for request/response formatting and configuration.
 */

/**
 * Get the Tailwind color class for an HTTP method
 * @param method - HTTP method (GET, POST, PUT, DELETE, PATCH)
 * @returns Tailwind text color class
 */
export const getMethodColor = (method: string): string => {
  switch (method?.toUpperCase()) {
    case 'GET': return 'text-green-600'
    case 'POST': return 'text-orange-600'
    case 'PUT': return 'text-blue-600'
    case 'DELETE': return 'text-red-600'
    case 'PATCH': return 'text-purple-600'
    default: return 'text-muted-foreground'
  }
}

/**
 * Format bytes into human-readable file size
 * @param bytes - Number of bytes
 * @returns Formatted size string (e.g., "1.5 KB", "2.3 MB")
 */
export const formatSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  const kb = bytes / 1024
  if (kb < 1024) return kb.toFixed(1) + ' KB'
  const mb = kb / 1024
  return mb.toFixed(1) + ' MB'
}

/**
 * Get the default state for a new request tab
 * @returns Default tab configuration object
 */
export const defaultTabState = () => ({
  params: [{ enabled: true, name: '', value: '' }],
  headers: [{ enabled: true, name: '', value: '' }],
  body: { type: 'application/json', content: '{\n  "key": "value"\n}' },
  auth: {
    type: 'bearer',
    active: true,
    bearerToken: '',
    basicUser: '',
    basicPass: '',
    apiKeyName: '',
    apiKeyValue: '',
    apiKeyLocation: 'header'
  },
  preflight: {
    enabled: false,
    method: 'POST',
    url: '',
    body: '',
    bodyType: 'application/json',
    bodyParams: [{ enabled: true, name: '', value: '' }],
    headers: [],
    cacheToken: true,
    cacheDuration: 'derived',
    cacheDurationKey: 'expires_in',
    cacheDurationUnit: 'seconds',
    tokenKey: 'access_token',
    tokenHeader: 'Authorization'
  },
  versions: [],
  response: {
    activeTab: 'body',
    status: 0,
    statusText: '',
    time: '0ms',
    size: '0 B',
    type: '',
    body: '',
    error: '',
    headers: [],
    requestHeaders: []
  },
  activeSubTab: 'params'
})
