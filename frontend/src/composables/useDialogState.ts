/**
 * Dialog State Management Composable
 *
 * Centralizes state management for all dialogs in the Services view.
 * Provides reactive boolean flags and helper functions for opening/closing dialogs.
 */

import { ref } from 'vue'

/**
 * Hook to manage dialog open/close state
 * @returns Object with dialog state refs and helper functions
 */
// Shared state refs (singleton pattern)
const isServiceDialogOpen = ref(false)
const isEndpointDialogOpen = ref(false)
const isCollectionDialogOpen = ref(false)
const isCollectionEndpointDialogOpen = ref(false)
const isSwaggerDialogOpen = ref(false)
const isShareDialogOpen = ref(false)
const isUnsafeDialogOpen = ref(false)
const isCurlDialogOpen = ref(false)

/**
 * Hook to manage dialog open/close state
 * @returns Object with dialog state refs and helper functions
 */
export const useDialogState = () => {

  // Helper functions
  const openServiceDialog = () => {
    isServiceDialogOpen.value = true
  }

  const closeServiceDialog = () => {
    isServiceDialogOpen.value = false
  }

  const openEndpointDialog = () => {
    isEndpointDialogOpen.value = true
  }

  const closeEndpointDialog = () => {
    isEndpointDialogOpen.value = false
  }

  const openCollectionDialog = () => {
    isCollectionDialogOpen.value = true
  }

  const closeCollectionDialog = () => {
    isCollectionDialogOpen.value = false
  }

  const openCollectionEndpointDialog = () => {
    isCollectionEndpointDialogOpen.value = true
  }

  const closeCollectionEndpointDialog = () => {
    isCollectionEndpointDialogOpen.value = false
  }

  const openSwaggerDialog = () => {
    isSwaggerDialogOpen.value = true
  }

  const closeSwaggerDialog = () => {
    isSwaggerDialogOpen.value = false
  }

  const openShareDialog = () => {
    isShareDialogOpen.value = true
  }

  const closeShareDialog = () => {
    isShareDialogOpen.value = false
  }

  const openUnsafeDialog = () => {
    isUnsafeDialogOpen.value = true
  }

  const closeUnsafeDialog = () => {
    isUnsafeDialogOpen.value = false
  }

  const openCurlDialog = () => {
    isCurlDialogOpen.value = true
  }

  const closeCurlDialog = () => {
    isCurlDialogOpen.value = false
  }

  return {
    // State
    isServiceDialogOpen,
    isEndpointDialogOpen,
    isCollectionDialogOpen,
    isCollectionEndpointDialogOpen,
    isSwaggerDialogOpen,
    isShareDialogOpen,
    isUnsafeDialogOpen,

    // Actions
    openServiceDialog,
    closeServiceDialog,
    openEndpointDialog,
    closeEndpointDialog,
    openCollectionDialog,
    closeCollectionDialog,
    openCollectionEndpointDialog,
    closeCollectionEndpointDialog,
    openSwaggerDialog,
    closeSwaggerDialog,
    openShareDialog,
    closeShareDialog,
    openUnsafeDialog,
    closeUnsafeDialog,
    isCurlDialogOpen,
    openCurlDialog,
    closeCurlDialog
  }
}
