import { ServiceGateway } from './service/gateway'
import { MockServiceGateway } from './service/mock-gateway'
import { CollectionGateway } from './collection/gateway'
import { MockCollectionGateway } from './collection/mock-gateway'
import { HistoryGateway } from './history/gateway'
import { MockHistoryGateway } from './history/mock-gateway'
import type { IServiceGateway } from '@/domains/service/ports'
import type { ICollectionGateway } from '@/domains/collection/ports'
import type { IHistoryGateway } from '@/domains/history/ports'

export class AdapterFactory {
    static getServiceGateway(): IServiceGateway {
        // @ts-ignore
        if (window.wails) {
            return new ServiceGateway()
        }
        return new MockServiceGateway()
    }

    static getCollectionGateway(): ICollectionGateway {
        // @ts-ignore
        if (window.wails) {
            return new CollectionGateway()
        }
        return new MockCollectionGateway()
    }

    static getHistoryGateway(): IHistoryGateway {
        // @ts-ignore
        if (window.wails) {
            return new HistoryGateway()
        }
        return new MockHistoryGateway()
    }
}
