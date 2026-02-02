import type { Service } from '@/types'
import type { ICollectionGateway } from './ports'

export class CollectionManager {
    constructor(private gateway: ICollectionGateway) { }

    async getAllCollections(): Promise<Service[]> {
        return this.gateway.loadCollections()
    }

    async saveCollections(collections: Service[]): Promise<Service[]> {
        return this.gateway.saveCollections(collections)
    }

    async addCollection(collections: Service[], collection: Service): Promise<Service[]> {
        const updated = [...collections, collection]
        return this.saveCollections(updated)
    }

    async updateCollection(collections: Service[], index: number, collection: Service): Promise<Service[]> {
        const updated = [...collections]
        updated[index] = collection
        return this.saveCollections(updated)
    }

    async deleteCollection(collections: Service[], index: number): Promise<Service[]> {
        const updated = [...collections]
        updated.splice(index, 1)
        return this.saveCollections(updated)
    }
}
