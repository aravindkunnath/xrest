import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ServiceManager } from '@/domains/service/manager'
import type { IServiceGateway } from '@/domains/service/ports'
import type { Service } from '@/types'

function createMockGateway(): { gateway: IServiceGateway; mocks: Record<string, ReturnType<typeof vi.fn>> } {
    const mocks = {
        importService: vi.fn(),
        importSwagger: vi.fn(),
        importCurl: vi.fn(),
        loadServices: vi.fn(),
        saveServices: vi.fn(),
        getGitStatus: vi.fn(),
        initGit: vi.fn(),
        syncGit: vi.fn(),
        pullGit: vi.fn(),
        pushGit: vi.fn(),
        commitGit: vi.fn(),
    }
    return { gateway: mocks as unknown as IServiceGateway, mocks }
}

const sampleService: Service = {
    id: 's1',
    name: 'Test Service',
    directory: '/some/path',
    isAuthenticated: false,
    endpoints: [],
    environments: [],
}

describe('ServiceManager', () => {
    let gateway: IServiceGateway
    let mocks: Record<string, ReturnType<typeof vi.fn>>

    beforeEach(() => {
        ({ gateway, mocks } = createMockGateway())
    })

    it('importService delegates to gateway', async () => {
        mocks.importService.mockResolvedValue(sampleService)

        const manager = new ServiceManager(gateway)
        const result = await manager.importService('/some/path')

        expect(mocks.importService).toHaveBeenCalledWith('/some/path')
        expect(mocks.importService).toHaveBeenCalledTimes(1)
        expect(result).toEqual(sampleService)
    })

    it('importSwagger delegates to gateway', async () => {
        mocks.importSwagger.mockResolvedValue(sampleService)

        const manager = new ServiceManager(gateway)
        const result = await manager.importSwagger('My API', '/path/to/spec.json')

        expect(mocks.importSwagger).toHaveBeenCalledWith('My API', '/path/to/spec.json')
        expect(mocks.importSwagger).toHaveBeenCalledTimes(1)
        expect(result).toEqual(sampleService)
    })

    it('importCurl delegates to gateway', async () => {
        const updated: Service = { ...sampleService, endpoints: [{ id: 'e1', serviceId: 's1', name: 'EP', method: 'GET', url: 'http://x', authenticated: false, authType: 'none', metadata: { version: '1', lastUpdated: 0 }, params: [], headers: [], body: '', preflight: { enabled: false, method: 'POST', url: '', body: '', headers: [], cacheToken: true, cacheDuration: 'derived', cacheDurationKey: 'expires_in', cacheDurationUnit: 'seconds', tokenKey: 'access_token', tokenHeader: 'Authorization' } }] }
        mocks.importCurl.mockResolvedValue(updated)

        const manager = new ServiceManager(gateway)
        const result = await manager.importCurl('s1', 'curl http://x')

        expect(mocks.importCurl).toHaveBeenCalledWith('s1', 'curl http://x')
        expect(mocks.importCurl).toHaveBeenCalledTimes(1)
        expect(result).toEqual(updated)
    })
})
