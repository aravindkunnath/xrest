import type { Service } from '@/types'

export interface ICollectionGateway {
    loadCollections(): Promise<Service[]>
    saveCollections(collections: Service[]): Promise<Service[]>
}
