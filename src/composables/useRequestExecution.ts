/**
 * Request Execution Composable
 *
 * Handles HTTP request execution including:
 * - Request validation
 * - Variable resolution for all request parts
 * - Unsafe environment warnings with countdown
 * - Request invocation via Tauri backend
 * - Response and error handling
 */

import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { invoke } from '@tauri-apps/api/core'
import { resolveVariables } from '@/lib/placeholders'
import { formatSize } from '@/lib/request-utils'
import { useEnvironmentVariables } from './useEnvironmentVariables'

// Shared state (singleton pattern)
const unsafeTabToProceed = ref<any>(null)
const unsafeCountdown = ref(10)
let unsafeTimer: any = null

/**
 * Hook to manage request execution
 * @param isUnsafeDialogOpen - Ref to unsafe dialog open state
 * @returns Object with request execution state and functions
 */
export const useRequestExecution = (isUnsafeDialogOpen: any) => {
  const isSending = ref(false)

  const { getTabVariables, isUnsafeEnv } = useEnvironmentVariables()

  /**
   * Proceed with an unsafe request after user confirmation
   * @param handleSendRequestFn - The send request function to call
   */
  const proceedWithUnsafeRequest = (handleSendRequestFn: Function) => {
    if (unsafeTimer) clearInterval(unsafeTimer)
    isUnsafeDialogOpen.value = false
    if (unsafeTabToProceed.value) {
      handleSendRequestFn(unsafeTabToProceed.value, true)
      unsafeTabToProceed.value = null
    }
  }

  /**
   * Cancel an unsafe request
   */
  const cancelUnsafeRequest = () => {
    if (unsafeTimer) clearInterval(unsafeTimer)
    isUnsafeDialogOpen.value = false
    unsafeTabToProceed.value = null
  }

  /**
   * Send an HTTP request for a tab
   * @param tab - Tab object with request configuration
   * @param skipWarning - Skip unsafe environment warning
   */
  const handleSendRequest = async (tab: any, skipWarning = false) => {
    if (isSending.value) return

    // Check for unsafe environment
    if (!skipWarning && isUnsafeEnv(tab)) {
      unsafeTabToProceed.value = tab
      isUnsafeDialogOpen.value = true
      unsafeCountdown.value = 10
      if (unsafeTimer) clearInterval(unsafeTimer)
      unsafeTimer = setInterval(() => {
        unsafeCountdown.value--
        if (unsafeCountdown.value <= 0) {
          cancelUnsafeRequest()
        }
      }, 1000)
      return
    }

    isSending.value = true

    try {
      // Basic validation
      if (!tab.url) {
        toast.error('URL is required')
        isSending.value = false
        return
      }

      const rawUrl = tab.url.trim()
      const vars = getTabVariables(tab)

      // Resolve all potential placeholders in the request
      const resolvedUrl = resolveVariables(rawUrl, vars)
      const resolvedParams = tab.params
        .filter((p: any) => p.enabled && p.name)
        .map((p: any) => ({
          name: resolveVariables(p.name, vars),
          value: resolveVariables(p.value, vars)
        }))

      const resolvedHeaders = tab.headers
        .filter((h: any) => h.enabled && h.name)
        .map((h: any) => ({
          name: resolveVariables(h.name, vars),
          value: resolveVariables(h.value, vars)
        }))

      const resolvedBody = resolveVariables(tab.body.content || '', vars)

      // Resolve Auth details if present
      const resolvedAuth = { ...tab.auth }
      if (resolvedAuth.bearerToken) resolvedAuth.bearerToken = resolveVariables(resolvedAuth.bearerToken, vars)
      if (resolvedAuth.basicUser) resolvedAuth.basicUser = resolveVariables(resolvedAuth.basicUser, vars)
      if (resolvedAuth.basicPass) resolvedAuth.basicPass = resolveVariables(resolvedAuth.basicPass, vars)
      if (resolvedAuth.apiKeyName) resolvedAuth.apiKeyName = resolveVariables(resolvedAuth.apiKeyName, vars)
      if (resolvedAuth.apiKeyValue) resolvedAuth.apiKeyValue = resolveVariables(resolvedAuth.apiKeyValue, vars)

      const payload = {
        ...tab,
        url: resolvedUrl,
        params: resolvedParams,
        headers: resolvedHeaders,
        body: { ...tab.body, content: resolvedBody },
        auth: resolvedAuth,
        serviceId: tab.serviceId,
        preflight: {
          ...tab.preflight,
          url: resolveVariables(tab.preflight.url || '', vars),
          body: resolveVariables(tab.preflight.body || '', vars),
        },
        variables: vars
      }

      const response: any = await invoke('send_request', { tab: payload })

      tab.response = {
        ...tab.response,
        status: response.status,
        statusText: response.statusText,
        time: `${response.timeElapsed}ms`,
        size: formatSize(response.size),
        body: response.body,
        headers: response.headers,
        error: '',
      }
    } catch (error) {
      console.error('Request failed:', error)
      tab.response = {
        ...tab.response,
        status: 0,
        statusText: 'Error',
        time: '0ms',
        size: '0 B',
        body: '',
        headers: [],
        error: String(error),
      }
    } finally {
      isSending.value = false
    }
  }

  return {
    isSending,
    unsafeTabToProceed,
    unsafeCountdown,
    handleSendRequest,
    proceedWithUnsafeRequest,
    cancelUnsafeRequest
  }
}
