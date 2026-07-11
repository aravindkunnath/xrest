import type { HistoryEntry } from '@/stores/history'

export interface IHistoryGateway {
    getHistory(limit: number, offset: number): Promise<HistoryEntry[]>
    addHistory(entry: Omit<HistoryEntry, 'id' | 'createdAt'>): Promise<HistoryEntry>
    clearHistory(): Promise<void>
}
