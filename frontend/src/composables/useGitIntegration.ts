/**
 * Git Integration Composable
 *
 * Manages git operations for service directories including:
 * - Fetching git status
 * - Initializing git repositories
 * - Syncing with remote repositories
 */

import { ref } from 'vue'
import { useServicesStore } from '@/stores/services'

const gitStatuses = ref<Record<string, any>>({})

/**
 * Hook to manage git integration for services
 * @returns Object with git status and operation functions
 */
export const useGitIntegration = () => {
  const servicesStore = useServicesStore()

  /**
   * Fetch git status for a service directory
   * @param serviceId - Service ID
   * @param directory - Directory path to check
   */
  const fetchGitStatus = async (serviceId: string, directory: string) => {
    const status = await servicesStore.getGitStatus(directory)
    if (status) {
      gitStatuses.value[serviceId] = status
    }
  }

  /**
   * Sync git repository with remote
   * @param serviceId - Service ID
   * @param directory - Directory path to sync
   */
  const handleSyncGit = async (serviceId: string, directory: string) => {
    await servicesStore.syncGit(directory)
    await fetchGitStatus(serviceId, directory)
  }

  /**
   * Initialize git repository in directory
   * @param serviceId - Service ID
   * @param directory - Directory path to initialize
   * @param gitUrl - Optional remote URL to set
   */
  const handleInitGit = async (serviceId: string, directory: string, gitUrl?: string) => {
    await servicesStore.initGit(directory, gitUrl)
    await fetchGitStatus(serviceId, directory)
  }

  /**
   * Pull git repository from remote
   * @param serviceId - Service ID
   * @param directory - Directory path to pull
   */
  const handlePullGit = async (serviceId: string, directory: string) => {
    await servicesStore.pullGit(directory)
    await fetchGitStatus(serviceId, directory)
  }

  /**
   * Push git repository to remote
   * @param serviceId - Service ID
   * @param directory - Directory path to push
   */
  const handlePushGit = async (serviceId: string, directory: string) => {
    await servicesStore.pushGit(directory)
    await fetchGitStatus(serviceId, directory)
  }

  /**
   * Commit changes to git repository
   * @param serviceId - Service ID
   * @param directory - Directory path to commit
   * @param message - Commit message
   */
  const handleCommitGit = async (serviceId: string, directory: string, message: string) => {
    await servicesStore.commitGit(directory, message)
    await fetchGitStatus(serviceId, directory)
  }

  return {
    gitStatuses,
    fetchGitStatus,
    handleSyncGit,
    handleInitGit,
    handlePullGit,
    handlePushGit,
    handleCommitGit
  }
}
