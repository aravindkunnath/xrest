import { describe, it, expect } from 'vitest'
import { AuthType, type Service } from '../index'

describe('Types Validation', () => {
    it('should create a valid service object', () => {
        const service: Service = {
            id: 's1',
            name: 'Test Service',
            environments: [
                {
                    name: 'Development',
                    isUnsafe: false,
                    variables: [
                        { name: 'BASE_URL', value: 'http://localhost:3000' }
                    ]
                }
            ],
            isAuthenticated: true,
            authType: AuthType.BEARER,
            endpoints: [
                {
                    id: 'e1',
                    serviceId: 's1',
                    name: 'Login',
                    url: '/login',
                    method: 'POST',
                    authenticated: true,
                    authType: 'bearer',
                    metadata: {
                        version: '1.0',
                        lastUpdated: Date.now()
                    },
                    params: [],
                    headers: [],
                    body: '{}',
                    preflight: {
                        enabled: false,
                        method: 'GET',
                        url: '',
                        body: '',
                        headers: [],
                        cacheToken: false,
                        cacheDuration: '',
                        cacheDurationKey: '',
                        cacheDurationUnit: '',
                        tokenKey: '',
                        tokenHeader: ''
                    }
                }
            ],
            directory: '/tmp'
        }

        expect(service.name).toBe('Test Service')
        expect(service.authType).toBe(AuthType.BEARER)
    })
})
