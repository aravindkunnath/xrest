import { TauriServiceGateway } from './service/tauri-gateway'
import { MockServiceGateway } from './service/mock-gateway'
import { TauriCollectionGateway } from './collection/tauri-gateway'
import { MockCollectionGateway } from './collection/mock-gateway'
import type { IServiceGateway } from '@/domains/service/ports'
import type { ICollectionGateway } from '@/domains/collection/ports'

export class AdapterFactory {
    static getServiceGateway(): IServiceGateway {
        // @ts-ignore
        if (window.__TAURI_INTERNALS__) {
            return new TauriServiceGateway()
        }
        return new MockServiceGateway()
    }

    static getCollectionGateway(): ICollectionGateway {
        // @ts-ignore
        if (window.__TAURI_INTERNALS__) {
            return new TauriCollectionGateway()
        }
        return new MockCollectionGateway()
    }
}
