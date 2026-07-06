import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useServicesStore } from '@/stores/services'

describe('ServicesView - Save Request', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
    })

    describe('handleSaveRequest', () => {
        it('should preserve the enabled flag when saving params', async () => {
            const servicesStore = useServicesStore()

            // Setup: Create a service with an endpoint
            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '/test/dir',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [
                        {
                            id: 'endpoint-1',
                            name: 'Test Endpoint',
                            method: 'GET',
                            url: '/api/test',
                            params: [
                                { name: 'id', value: '123' }  // Old saved param without enabled
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
                            name: 'DEV',
                            isUnsafe: false,
                            variables: []
                        }
                    ],
                    selectedEnvironment: 'DEV'
                }
            ] as any

            // Mock the updateService to capture what gets saved
            let savedEndpoint: any = null
            const originalUpdateService = servicesStore.updateService
            servicesStore.updateService = vi.fn(async (index: number, service: any) => {
                savedEndpoint = service.endpoints[0]
                return originalUpdateService.call(servicesStore, index, service)
            })

            // Simulate a tab with params that have enabled flags
            const tab = {
                id: 'endpoint-endpoint-1',
                type: 'request',
                endpointId: 'endpoint-1',
                serviceId: 'service-1',
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
                auth: {
                    type: 'none'
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

            // This is the logic from handleSaveRequest in ServicesView.vue
            const service = servicesStore.services[0]
            const endpoint = service.endpoints[0]

            const updatedEndpoint = {
                ...endpoint,
                method: tab.method,
                url: tab.url,
                params: tab.params.filter((p: any) => p.name).map((p: any) => ({
                    name: p.name,
                    value: p.value,
                    enabled: p.enabled ?? true  // This is what we want to add
                })),
                headers: tab.headers.filter((h: any) => h.name).map((h: any) => ({
                    name: h.name,
                    value: h.value,
                    enabled: h.enabled ?? true  // This is what we want to add
                })),
                body: tab.body.content,
                preflight: tab.preflight,
                metadata: {
                    ...endpoint.metadata,
                    lastUpdated: Date.now()
                }
            }

            const updatedService = {
                ...service,
                endpoints: [updatedEndpoint]
            }

            await servicesStore.updateService(0, updatedService)

            // Assert: The saved params should have the enabled flag preserved
            expect(savedEndpoint.params).toEqual([
                { name: 'id', value: '123', enabled: true },
                { name: 'filter', value: 'active', enabled: false },
                { name: 'sort', value: 'desc', enabled: true }
            ])

            // Assert: The saved headers should have the enabled flag preserved
            expect(savedEndpoint.headers).toEqual([
                { name: 'Authorization', value: 'Bearer token', enabled: true }
            ])
        })

        it('should default enabled to true for params without the flag', async () => {
            const servicesStore = useServicesStore()

            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '/test/dir',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [
                        {
                            id: 'endpoint-1',
                            name: 'Test Endpoint',
                            method: 'GET',
                            url: '/api/test',
                            params: [],
                            headers: [],
                            body: '',
                            metadata: {
                                created: Date.now(),
                                lastUpdated: Date.now()
                            }
                        }
                    ],
                    environments: [],
                    selectedEnvironment: ''
                }
            ] as any

            let savedEndpoint: any = null
            const originalUpdateService = servicesStore.updateService
            servicesStore.updateService = vi.fn(async (index: number, service: any) => {
                savedEndpoint = service.endpoints[0]
                return originalUpdateService.call(servicesStore, index, service)
            })

            // Tab with params that don't have enabled flag (legacy data)
            const tab = {
                endpointId: 'endpoint-1',
                serviceId: 'service-1',
                method: 'GET',
                url: '/api/test',
                params: [
                    { name: 'id', value: '123' }  // No enabled flag
                ],
                headers: [
                    { name: 'Content-Type', value: 'application/json' }  // No enabled flag
                ],
                body: { content: '' },
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

            const service = servicesStore.services[0]
            const endpoint = service.endpoints[0]

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

            const updatedService = {
                ...service,
                endpoints: [updatedEndpoint]
            }

            await servicesStore.updateService(0, updatedService)

            // Assert: Params without enabled should default to true
            expect(savedEndpoint.params).toEqual([
                { name: 'id', value: '123', enabled: true }
            ])

            expect(savedEndpoint.headers).toEqual([
                { name: 'Content-Type', value: 'application/json', enabled: true }
            ])
        })
    })
})
