import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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
            const data = await invoke<HistoryEntry[]>('get_history', { limit, offset })
            history.value = data
        } catch (err: any) {
            error.value = err.toString()
            console.error('Failed to fetch history:', err)
        } finally {
            isLoading.value = false
        }
    }

    async function clearHistory() {
        isLoading.value = true
        try {
            await invoke('clear_history')
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
        clearHistory
    }
})
