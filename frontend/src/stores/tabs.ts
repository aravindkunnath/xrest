import { defineStore } from 'pinia'
import { ref, watch, nextTick } from 'vue'
import { defaultTabState } from '@/lib/request-utils'

export interface Tab {
    id: string
    title: string
    method?: string
    url?: string
    type: 'request' | 'settings'
    isEdited?: boolean
    [key: string]: any
}

export const useTabsStore = defineStore('tabs', () => {
    const activeTab = ref('tab-1')
    const tabs = ref<Tab[]>([])
    const tabSnapshots = new Map<string, string>()
    const isInitialized = ref(false)

    const getTabSnapshot = (tab: Tab): string => {
        const { response, isEdited, versions, activeSubTab, ...savedState } = tab
        return JSON.stringify(savedState)
    }

    const updateTabSnapshot = (tab: Tab): void => {
        tabSnapshots.set(tab.id, getTabSnapshot(tab))
        tab.isEdited = false
    }

    // Helper to strip heavy/transient data for persistence
    const getPersistentState = (tab: Tab) => {
        const { response, isEdited, versions, activeSubTab, ...persistent } = tab
        return persistent
    }

    const saveTabState = async (): Promise<void> => {
        try {
            // Stripping responses before sending to IPC saves massive memory/bandwidth
            const tabsToSave = tabs.value.map(getPersistentState)

            localStorage.setItem('xrest_min_tabs_state', JSON.stringify({
                activeTabId: activeTab.value,
                tabs: tabsToSave
            }))
        } catch (error) {
            console.error('Failed to save tab state:', error)
        }
    }

    // Use a simple timer for debouncing to avoid adding full lodash dependency
    let saveTimer: any = null
    const debouncedSave = () => {
        if (saveTimer) clearTimeout(saveTimer)
        saveTimer = setTimeout(saveTabState, 1000)
    }

    const addTab = (overrides: Partial<Tab> = {}): void => {
        const newId = overrides.id || `tab-${Date.now()}`
        const newTab: Tab = {
            id: newId,
            title: 'New Request',
            method: 'GET',
            url: 'https://api.example.com/',
            type: 'request',
            isEdited: false,
            ...defaultTabState(),
            ...overrides
        }
        tabs.value.push(newTab)

        nextTick(() => {
            updateTabSnapshot(newTab)
        })

        setTimeout(() => {
            activeTab.value = newId
        }, 0)
    }

    const closeTab = async (id: string): Promise<void> => {
        const tab = tabs.value.find(t => t.id === id)
        if (tab?.isEdited) {
            // @ts-ignore
            const { Dialogs } = await import('@wailsio/runtime')
            const result = await Dialogs.Question({
                Title: 'Unsaved Changes',
                Message: `You have unsaved changes in "${tab.title}". Are you sure you want to close it?`,
                Buttons: [
                    { Label: 'Yes', IsDefault: true },
                    { Label: 'No', IsCancel: true }
                ]
            })
            const confirmation = result === 'Yes'
            if (!confirmation) return
        }

        tabs.value = tabs.value.filter(t => t.id !== id)
        tabSnapshots.delete(id)
        if (activeTab.value === id && tabs.value.length > 0) {
            activeTab.value = tabs.value[0].id
        }
    }

    const loadTabs = async (): Promise<void> => {
        if (isInitialized.value) return

        try {
            let savedState: any = null
            const saved = localStorage.getItem('xrest_min_tabs_state')
            if (saved) {
                savedState = JSON.parse(saved)
            }

            if (savedState && savedState.tabs && savedState.tabs.length > 0) {
                tabs.value = savedState.tabs.map((t: any) => ({
                    ...t,
                    response: defaultTabState().response // Start with empty response to save memory
                }))
                activeTab.value = savedState.activeTabId

                nextTick(() => {
                    tabs.value.forEach(tab => {
                        updateTabSnapshot(tab)
                    })
                })
            } else if (tabs.value.length === 0) {
                // Add a default tab if none exists
                addTab()
            }
            isInitialized.value = true
        } catch (error) {
            console.error('Failed to load tab state:', error)
            if (tabs.value.length === 0) addTab()
        }
    }

    watch(tabs, (newTabs) => {
        newTabs.forEach(tab => {
            const snapshot = tabSnapshots.get(tab.id)
            if (snapshot) {
                const current = JSON.stringify(getPersistentState(tab))
                tab.isEdited = snapshot !== current
            }
        })
        debouncedSave()
    }, { deep: true })

    watch(activeTab, () => {
        debouncedSave()
    })

    return {
        tabs,
        activeTab,
        isInitialized,
        addTab,
        closeTab,
        updateTabSnapshot,
        saveTabState,
        loadTabs
    }
})
