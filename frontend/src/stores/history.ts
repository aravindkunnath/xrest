import { defineStore } from 'pinia'
import { ref } from 'vue'
import { AdapterFactory } from '@/infrastructure/adapter-factory'

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
    const gateway = AdapterFactory.getHistoryGateway()

    async function fetchHistory(limit: number = 50, offset: number = 0) {
        isLoading.value = true
        try {
            const entries = await gateway.getHistory(limit, offset)
            history.value = entries
        } catch (err: any) {
            error.value = err.toString()
            console.error('Failed to fetch history:', err)
        } finally {
            isLoading.value = false
        }
    }

    async function addHistoryEntry(entry: Omit<HistoryEntry, 'id' | 'createdAt'>) {
        try {
            const newEntry = await gateway.addHistory(entry)
            history.value = [newEntry, ...history.value]
            if (history.value.length > 100) {
                history.value = history.value.slice(0, 100)
            }
        } catch (err: any) {
            console.error('Failed to add history entry:', err)
        }
    }

    async function clearHistory() {
        isLoading.value = true
        try {
            await gateway.clearHistory()
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
