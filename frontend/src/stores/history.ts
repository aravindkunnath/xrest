import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface HistoryEntry {
    id: string
    serviceId: string | null
    endpointId: string | null
    method: string
    url: string
    requestHeaders: { name: string; value: string }[]
    requestBody: string
    responseStatus: number
    responseStatusText: string
    responseHeaders: { name: string; value: string }[]
    responseBody: string
    timeElapsed: number
    size: number
    createdAt: string
}

export const useHistoryStore = defineStore('history', () => {
    const history = ref<HistoryEntry[]>([])
    const isLoading = ref(false)
    const error = ref<string | null>(null)

    async function fetchHistory(limit: number = 50, offset: number = 0) {
        isLoading.value = true
        try {
            const saved = localStorage.getItem('xrest_history')
            if (saved) {
                const allEntries = JSON.parse(saved) as HistoryEntry[]
                history.value = allEntries.slice(offset, offset + limit)
            } else {
                history.value = []
            }
        } catch (err: any) {
            error.value = err.toString()
            console.error('Failed to fetch history:', err)
        } finally {
            isLoading.value = false
        }
    }

    async function addHistoryEntry(entry: Omit<HistoryEntry, 'id' | 'createdAt'>) {
        try {
            const newEntry: HistoryEntry = {
                ...entry,
                id: `history-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
                createdAt: new Date().toISOString()
            }
            const saved = localStorage.getItem('xrest_history')
            const allEntries = saved ? (JSON.parse(saved) as HistoryEntry[]) : []
            allEntries.unshift(newEntry)
            if (allEntries.length > 100) {
                allEntries.length = 100
            }
            localStorage.setItem('xrest_history', JSON.stringify(allEntries))
            history.value = allEntries
        } catch (err: any) {
            console.error('Failed to add history entry:', err)
        }
    }

    async function clearHistory() {
        isLoading.value = true
        try {
            localStorage.removeItem('xrest_history')
            history.value = []
        } catch (err: any) {
            error.value = err.toString()
            console.error('Failed to clear history:', err)
        } finally {
            isLoading.value = false
        }
    }

    return {
        history,
        isLoading,
        error,
        fetchHistory,
        addHistoryEntry,
        clearHistory
    }
})
