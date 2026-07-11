/**
 * Environment Variables Composable
 *
 * Manages environment variables across services, including:
 * - Active variable computation per service
 * - Variable resolution for tabs
 * - Unsafe environment detection
 * - Parameter chaining (params can reference env vars)
 */

import { computed } from 'vue'
import { useServicesStore } from '@/stores/services'
import { useCollectionsStore } from '@/stores/collections'
import { resolveVariables } from '@/lib/placeholders'

/**
 * Hook to manage environment variables across services and collections
 * @returns Object with computed variables and helper functions
 */
export const useEnvironmentVariables = () => {
  const servicesStore = useServicesStore()
  const collectionsStore = useCollectionsStore()

  /**
   * Compute all active variables for each service and collection
   * Returns a map of id -> variables
   */
  const allActiveVariables = computed(() => {
    const map: Record<string, Record<string, string>> = {}

    // Add services variables
    servicesStore.services.forEach(service => {
      const envName = service.selectedEnvironment || (service.environments.length > 0 ? service.environments[0].name : null)
      const env = service.environments.find(e => e.name === envName)
      const vars: Record<string, string> = {}
      if (env) {
        env.variables.forEach(v => {
          if (v.name) {
            if (v.secretKey) {
              vars[v.name] = `{{secret.${v.secretKey}}}`
            } else {
              vars[v.name] = v.value || ''
            }
          }
        })
      }
      map[service.id] = vars
    })

    // Add collections variables
    collectionsStore.collections.forEach(collection => {
      const envName = collection.selectedEnvironment || (collection.environments.length > 0 ? collection.environments[0].name : null)
      const env = collection.environments.find(e => e.name === envName)
      const vars: Record<string, string> = {}
      if (env) {
        env.variables.forEach(v => {
          if (v.name) {
            if (v.secretKey) {
              vars[v.name] = `{{secret.${v.secretKey}}}`
            } else {
              vars[v.name] = v.value || ''
            }
          }
        })
      }
      map[collection.id] = vars
    })

    return map
  })

  /**
   * Compute active environment name for each service and collection
   * Returns a map of id -> environmentName
   */
  const activeEnvironments = computed(() => {
    const map: Record<string, string> = {}

    servicesStore.services.forEach(service => {
      const envName = service.selectedEnvironment || service.environments[0]?.name
      map[service.id] = envName || 'DEFAULT'
    })

    collectionsStore.collections.forEach(collection => {
      const envName = collection.selectedEnvironment || collection.environments[0]?.name
      map[collection.id] = envName || 'DEFAULT'
    })

    return map
  })

  /**
   * Get all variables available for a specific tab
   * Includes environment variables + enabled parameters (for chaining)
   * @param tab - Tab object
   * @returns Object with all available variables
   */
  const getTabVariables = (tab: any) => {
    if (!tab || tab.type === 'settings') return {}
    const envVars = allActiveVariables.value[tab.serviceId] || {}
    const paramVars: Record<string, string> = {}

    if (tab.params) {
      tab.params.forEach((p: any) => {
        if (p.name) {
          // Resolve parameter value using environment variables for chaining
          paramVars[p.name] = resolveVariables(p.value, envVars)
        }
      })
    }

    return { ...envVars, ...paramVars }
  }

  /**
   * Check if a tab is targeting an unsafe environment
   * @param tab - Tab object
   * @returns true if the tab's environment is marked as unsafe
   */
  const isUnsafeEnv = (tab: any): boolean => {
    if (!tab || tab.type === 'settings' || !tab.serviceId) return false

    const service = servicesStore.services.find(s => s.id === tab.serviceId)
    if (service) {
      const envName = service.selectedEnvironment || service.environments[0]?.name
      const env = service.environments.find(e => e.name === envName)
      return env?.isUnsafe || false
    }

    const collection = collectionsStore.collections.find(c => c.id === tab.serviceId)
    if (collection) {
      const envName = collection.selectedEnvironment || collection.environments[0]?.name
      const env = collection.environments.find(e => e.name === envName)
      return env?.isUnsafe || false
    }

    return false
  }

  /**
   * Get the environment name for a tab
   * @param tab - Tab object
   * @returns Environment name string
   */
  const getEnvName = (tab: any): string => {
    if (!tab || !tab.serviceId) return ''

    const service = servicesStore.services.find(s => s.id === tab.serviceId)
    if (service) {
      return service.selectedEnvironment || service.environments[0]?.name || ''
    }

    const collection = collectionsStore.collections.find(c => c.id === tab.serviceId)
    if (collection) {
      return collection.selectedEnvironment || collection.environments[0]?.name || ''
    }

    return ''
  }

  return {
    allActiveVariables,
    activeEnvironments,
    getTabVariables,
    isUnsafeEnv,
    getEnvName
  }
}
