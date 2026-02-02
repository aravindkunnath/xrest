import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEnvironmentVariables } from '../useEnvironmentVariables'
import { useServicesStore } from '@/stores/services'
import { useCollectionsStore } from '@/stores/collections'

describe('useEnvironmentVariables', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
    })

    describe('getTabVariables', () => {
        it('should return environment variables for a tab with serviceId', () => {
            const servicesStore = useServicesStore()
            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'DEV',
                            isUnsafe: false,
                            variables: [
                                { name: 'BASE_URL', value: 'https://api.dev.example.com' },
                                { name: 'API_KEY', value: 'dev-key-123' }
                            ]
                        }
                    ],
                    selectedEnvironment: 'DEV'
                }
            ] as any

            const { getTabVariables } = useEnvironmentVariables()

            const tab = {
                id: 'tab-1',
                type: 'request',
                serviceId: 'service-1',
                params: []
            }

            const variables = getTabVariables(tab)

            expect(variables).toEqual({
                BASE_URL: 'https://api.dev.example.com',
                API_KEY: 'dev-key-123'
            })
        })

        it('should include param variables regardless of enabled state', () => {
            const servicesStore = useServicesStore()
            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'DEV',
                            isUnsafe: false,
                            variables: [
                                { name: 'BASE_URL', value: 'https://api.dev.example.com' }
                            ]
                        }
                    ],
                    selectedEnvironment: 'DEV'
                }
            ] as any

            const { getTabVariables } = useEnvironmentVariables()

            const tab = {
                id: 'tab-1',
                type: 'request',
                serviceId: 'service-1',
                params: [
                    { name: 'userId', value: '123', enabled: true },
                    { name: 'token', value: 'abc', enabled: false },
                    { name: 'filter', value: 'active' } // No enabled property
                ]
            }

            const variables = getTabVariables(tab)

            // All params should be included for variable interpolation,
            // regardless of enabled state
            expect(variables).toEqual({
                BASE_URL: 'https://api.dev.example.com',
                userId: '123',
                token: 'abc',
                filter: 'active'
            })
        })

        it('should resolve nested variable references in params', () => {
            const servicesStore = useServicesStore()
            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'DEV',
                            isUnsafe: false,
                            variables: [
                                { name: 'ENV', value: 'development' },
                                { name: 'VERSION', value: 'v1' }
                            ]
                        }
                    ],
                    selectedEnvironment: 'DEV'
                }
            ] as any

            const { getTabVariables } = useEnvironmentVariables()

            const tab = {
                id: 'tab-1',
                type: 'request',
                serviceId: 'service-1',
                params: [
                    { name: 'environment', value: '{{ENV}}' },
                    { name: 'apiVersion', value: '{{VERSION}}' }
                ]
            }

            const variables = getTabVariables(tab)

            // Params should have their variables resolved
            expect(variables).toEqual({
                ENV: 'development',
                VERSION: 'v1',
                environment: 'development',
                apiVersion: 'v1'
            })
        })

        it('should work with tabs loaded from saved state without enabled property', () => {
            const servicesStore = useServicesStore()
            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Test Service',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'PROD',
                            isUnsafe: true,
                            variables: [
                                { name: 'BASE_URL', value: 'https://api.example.com' },
                                { name: 'TIMEOUT', value: '5000' }
                            ]
                        }
                    ],
                    selectedEnvironment: 'PROD'
                }
            ] as any

            const { getTabVariables } = useEnvironmentVariables()

            // Simulate a tab loaded from saved state where params might not have 'enabled'
            const savedTab = {
                id: 'endpoint-123',
                type: 'request',
                serviceId: 'service-1',
                title: 'Get User',
                method: 'GET',
                url: '{{BASE_URL}}/users',
                params: [
                    { name: 'limit', value: '10' },
                    { name: 'offset', value: '0' }
                ],
                headers: [],
                body: { type: 'application/json', content: '' }
            }

            const variables = getTabVariables(savedTab)

            // Should include both environment variables and param variables
            expect(variables).toEqual({
                BASE_URL: 'https://api.example.com',
                TIMEOUT: '5000',
                limit: '10',
                offset: '0'
            })
        })

        it('should return empty object for settings tabs', () => {
            const { getTabVariables } = useEnvironmentVariables()

            const settingsTab = {
                id: 'settings-1',
                type: 'settings',
                serviceId: 'service-1'
            }

            const variables = getTabVariables(settingsTab)

            expect(variables).toEqual({})
        })

        it('should return param variables even for tabs without serviceId', () => {
            const { getTabVariables } = useEnvironmentVariables()

            const tab = {
                id: 'tab-1',
                type: 'request',
                params: [{ name: 'test', value: 'value' }]
            }

            const variables = getTabVariables(tab)

            // Even without serviceId, param variables should be available
            expect(variables).toEqual({ test: 'value' })
        })

        it('should work with collection tabs', () => {
            const collectionsStore = useCollectionsStore()
            collectionsStore.collections = [
                {
                    id: 'collection-1',
                    name: 'Test Collection',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'GLOBAL',
                            isUnsafe: false,
                            variables: [
                                { name: 'BASE_URL', value: 'https://api.example.com' },
                                { name: 'USER_ID', value: '42' }
                            ]
                        }
                    ],
                    selectedEnvironment: 'GLOBAL'
                }
            ] as any

            const { getTabVariables } = useEnvironmentVariables()

            const tab = {
                id: 'tab-1',
                type: 'request',
                serviceId: 'collection-1',
                params: [
                    { name: 'id', value: '{{USER_ID}}' }
                ]
            }

            const variables = getTabVariables(tab)

            expect(variables).toEqual({
                BASE_URL: 'https://api.example.com',
                USER_ID: '42',
                id: '42'
            })
        })
    })

    describe('allActiveVariables', () => {
        it('should compute variables for all services and collections', () => {
            const servicesStore = useServicesStore()
            const collectionsStore = useCollectionsStore()

            servicesStore.services = [
                {
                    id: 'service-1',
                    name: 'Service 1',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'DEV',
                            isUnsafe: false,
                            variables: [{ name: 'VAR1', value: 'value1' }]
                        }
                    ],
                    selectedEnvironment: 'DEV'
                }
            ] as any

            collectionsStore.collections = [
                {
                    id: 'collection-1',
                    name: 'Collection 1',
                    directory: '',
                    isAuthenticated: false,
                    authType: 'none',
                    endpoints: [],
                    environments: [
                        {
                            name: 'GLOBAL',
                            isUnsafe: false,
                            variables: [{ name: 'VAR2', value: 'value2' }]
                        }
                    ],
                    selectedEnvironment: 'GLOBAL'
                }
            ] as any

            const { allActiveVariables } = useEnvironmentVariables()

            expect(allActiveVariables.value).toEqual({
                'service-1': { VAR1: 'value1' },
                'collection-1': { VAR2: 'value2' }
            })
        })
    })
})
