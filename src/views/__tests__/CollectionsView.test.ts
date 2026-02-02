import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCollectionsStore } from '@/stores/collections'

describe('CollectionsView - Save Request', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
    })

    describe('handleSaveRequest', () => {
        it('should preserve the enabled flag when saving params', async () => {
            const collectionsStore = useCollectionsStore()

            collectionsStore.collections = [
                {
                    id: 'collection-1',
                    name: 'Test Collection',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [
                        {
                            id: 'endpoint-1',
                            name: 'Test Endpoint',
                            method: 'GET',
                            url: '/api/test',
                            params: [
                                { name: 'id', value: '123' }
                            ],
                            headers: [],
                            body: '',
                            metadata: {
                                created: Date.now(),
                                lastUpdated: Date.now()
                            }
                        }
                    ],
                    environments: [
                        {
                            name: 'GLOBAL',
                            isUnsafe: false,
                            variables: []
                        }
                    ],
                    selectedEnvironment: 'GLOBAL'
                }
            ] as any

            let savedEndpoint: any = null
            const originalUpdateCollection = collectionsStore.updateCollection
            collectionsStore.updateCollection = vi.fn(async (index: number, collection: any) => {
                savedEndpoint = collection.endpoints[0]
                return originalUpdateCollection.call(collectionsStore, index, collection)
            })

            const tab = {
                id: 'endpoint-endpoint-1',
                type: 'request',
                endpointId: 'endpoint-1',
                serviceId: 'collection-1',
                title: 'Test Endpoint',
                method: 'GET',
                url: '/api/test',
                params: [
                    { name: 'id', value: '123', enabled: true },
                    { name: 'filter', value: 'active', enabled: false },
                    { name: 'sort', value: 'desc', enabled: true }
                ],
                headers: [
                    { name: 'Authorization', value: 'Bearer token', enabled: true }
                ],
                body: {
                    type: 'application/json',
                    content: ''
                },
                preflight: {
                    enabled: false,
                    method: 'POST',
                    url: '',
                    body: '',
                    headers: [],
                    cacheToken: true,
                    cacheDuration: 'derived',
                    cacheDurationKey: 'expires_in',
                    cacheDurationUnit: 'seconds',
                    tokenKey: 'access_token',
                    tokenHeader: 'Authorization'
                }
            }

            const collection = collectionsStore.collections[0]
            const endpoint = collection.endpoints[0]

            const updatedEndpoint = {
                ...endpoint,
                method: tab.method,
                url: tab.url,
                params: tab.params.filter((p: any) => p.name).map((p: any) => ({
                    name: p.name,
                    value: p.value,
                    enabled: p.enabled ?? true
                })),
                headers: tab.headers.filter((h: any) => h.name).map((h: any) => ({
                    name: h.name,
                    value: h.value,
                    enabled: h.enabled ?? true
                })),
                body: tab.body.content,
                preflight: tab.preflight,
                metadata: {
                    ...endpoint.metadata,
                    lastUpdated: Date.now()
                }
            }

            const updatedCollection = {
                ...collection,
                endpoints: [updatedEndpoint]
            }

            await collectionsStore.updateCollection(0, updatedCollection)

            expect(savedEndpoint.params).toEqual([
                { name: 'id', value: '123', enabled: true },
                { name: 'filter', value: 'active', enabled: false },
                { name: 'sort', value: 'desc', enabled: true }
            ])

            expect(savedEndpoint.headers).toEqual([
                { name: 'Authorization', value: 'Bearer token', enabled: true }
            ])
        })
    })
})
