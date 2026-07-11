import { HistoryGateway as WailsHistoryGateway } from '../../../bindings/xrest/cmd/wails'
import type { IHistoryGateway } from '@/domains/history/ports'
import type { HistoryEntry } from '@/stores/history'

export class HistoryGateway implements IHistoryGateway {
    async getHistory(limit: number, offset: number): Promise<HistoryEntry[]> {
        const result = await WailsHistoryGateway.GetHistory(limit, offset)
        return (result as unknown as HistoryEntry[]) || []
    }

    async addHistory(entry: Omit<HistoryEntry, 'id' | 'createdAt'>): Promise<HistoryEntry> {
        const result = await WailsHistoryGateway.AddHistory(entry as any)
        return result as unknown as HistoryEntry
    }

    async clearHistory(): Promise<void> {
        await WailsHistoryGateway.ClearHistory()
    }
}
