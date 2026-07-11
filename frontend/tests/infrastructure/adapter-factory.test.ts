import { describe, it, expect, afterEach } from 'vitest'
import { AdapterFactory } from '@/infrastructure/adapter-factory'
import { ServiceGateway } from '@/infrastructure/service/gateway'
import { MockServiceGateway } from '@/infrastructure/service/mock-gateway'
import { CollectionGateway } from '@/infrastructure/collection/gateway'
import { MockCollectionGateway } from '@/infrastructure/collection/mock-gateway'

describe('AdapterFactory', () => {
    const originalWails = (window as any).wails

    afterEach(() => {
        if (originalWails === undefined) {
            delete (window as any).wails
        } else {
            (window as any).wails = originalWails
        }
    })

    it('should return mock gateways when window.wails is not defined', () => {
        delete (window as any).wails

        const serviceGateway = AdapterFactory.getServiceGateway()
        const collectionGateway = AdapterFactory.getCollectionGateway()

        expect(serviceGateway).toBeInstanceOf(MockServiceGateway)
        expect(collectionGateway).toBeInstanceOf(MockCollectionGateway)
    })

    it('should return Wails gateways when window.wails is defined', () => {
        (window as any).wails = {}

        const serviceGateway = AdapterFactory.getServiceGateway()
        const collectionGateway = AdapterFactory.getCollectionGateway()

        expect(serviceGateway).toBeInstanceOf(ServiceGateway)
        expect(collectionGateway).toBeInstanceOf(CollectionGateway)
    })
})
