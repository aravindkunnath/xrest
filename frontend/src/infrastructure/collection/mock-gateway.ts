import type { ICollectionGateway } from '@/domains/collection/ports'
import type { Service } from '@/types'

export class MockCollectionGateway implements ICollectionGateway {
    private collections: Service[] = []

    constructor() {
        const stored = localStorage.getItem('mock_collections')
        if (stored) {
            this.collections = JSON.parse(stored)
        }
    }

    async loadCollections(): Promise<Service[]> {
        return [...this.collections]
    }

    async saveCollections(collections: Service[]): Promise<Service[]> {
        this.collections = collections
        this.saveToStorage()
        return [...this.collections]
    }

    private saveToStorage() {
        localStorage.setItem('mock_collections', JSON.stringify(this.collections))
    }
}
