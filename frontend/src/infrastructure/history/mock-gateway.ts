import type { IHistoryGateway } from '@/domains/history/ports'
import type { HistoryEntry } from '@/stores/history'

export class MockHistoryGateway implements IHistoryGateway {
    async getHistory(limit: number, offset: number): Promise<HistoryEntry[]> {
        const saved = localStorage.getItem('xrest_history')
        if (saved) {
            const allEntries = JSON.parse(saved) as HistoryEntry[]
            return allEntries.slice(offset, offset + limit)
        }
        return []
    }

    async addHistory(entry: Omit<HistoryEntry, 'id' | 'createdAt'>): Promise<HistoryEntry> {
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
        return newEntry
    }

    async clearHistory(): Promise<void> {
        localStorage.removeItem('xrest_history')
    }
}
