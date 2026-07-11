/**
 * Tab Manager Composable
 *
 * Manages tab lifecycle and state including:
 * - Tab creation and closing
 * - Edit detection via snapshots
 * - Tab state persistence to Tauri backend
 * - Confirmation dialogs for unsaved changes
 */

import { storeToRefs } from 'pinia'
import { useTabsStore } from '@/stores/tabs'

export const useTabManager = () => {
  const tabsStore = useTabsStore()
  const { tabs, activeTab } = storeToRefs(tabsStore)

  return {
    tabs,
    activeTab,
    addTab: tabsStore.addTab,
    closeTab: tabsStore.closeTab,
    updateTabSnapshot: tabsStore.updateTabSnapshot,
    saveTabState: tabsStore.saveTabState,
    initializeTabsFromSavedState: tabsStore.loadTabs
  }
}
