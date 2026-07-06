import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockGateway = {
    importCurl: vi.fn(),
    loadServices: vi.fn(),
    saveServices: vi.fn(),
    getGitStatus: vi.fn(),
    initGit: vi.fn(),
    syncGit: vi.fn(),
    importService: vi.fn(),
}

vi.mock('@/infrastructure/adapter-factory', () => ({
    AdapterFactory: {
        getServiceGateway: vi.fn(() => mockGateway)
    }
}))

import { useServicesStore } from '@/stores/services'

describe('Services Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        vi.clearAllMocks()
    })

    it('should import curl and update service in store', async () => {
        const store = useServicesStore()
        const serviceId = 's1'
        const curlCommand = 'curl https://api.example.com'
        const updatedService = {
            id: serviceId,
            name: 'Test Service',
            endpoints: [{ id: 'e1', name: 'New Endpoint' }]
        }

        // Mock initial state
        store.services = [{ id: serviceId, name: 'Test Service', endpoints: [] }] as any

        vi.mocked(mockGateway.importCurl).mockResolvedValue(updatedService)

        const result = await store.importCurl(serviceId, curlCommand)

        expect(mockGateway.importCurl).toHaveBeenCalledWith(serviceId, curlCommand)
        expect(result).toEqual(updatedService)
        expect(store.services[0].endpoints.length).toBe(1)
    })

    it('should handle import curl error', async () => {
        const store = useServicesStore()
        vi.mocked(mockGateway.importCurl).mockRejectedValue(new Error('Parse error'))

        const result = await store.importCurl('s1', 'invalid curl')

        expect(result).toBeNull()
    })
})
