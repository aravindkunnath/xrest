import { CollectionGateway as WailsCollectionGateway } from '../../../bindings/xrest/cmd/wails'
import type { ICollectionGateway } from '@/domains/collection/ports'
import type { Service } from '@/types'

export class CollectionGateway implements ICollectionGateway {
    async loadCollections(): Promise<Service[]> {
        const result = await WailsCollectionGateway.LoadCollections()
        return (result as unknown as Service[]) || []
    }

    async saveCollections(collections: Service[]): Promise<Service[]> {
        const result = await WailsCollectionGateway.SaveCollections(collections as any)
        return (result as unknown as Service[]) || []
    }
}
