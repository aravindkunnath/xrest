import { invoke } from '@tauri-apps/api/core'
import type { ICollectionGateway } from '@/domains/collection/ports'
import type { Service } from '@/types'

export class TauriCollectionGateway implements ICollectionGateway {
    async loadCollections(): Promise<Service[]> {
        return invoke<Service[]>('get_collections')
    }

    async saveCollections(collections: Service[]): Promise<Service[]> {
        return invoke<Service[]>('save_collections', { collections })
    }
}
