import { describe, it, expect, beforeEach } from 'vitest'
import { MockCollectionGateway } from "@/infrastructure/collection/mock-gateway"
import { MockServiceGateway } from "@/infrastructure/service/mock-gateway"
import { MockVersionGateway } from "@/infrastructure/versions/mock-gateway"
import type { Service } from "@/types"

describe('Mock Gateways', () => {
    beforeEach(() => {
        localStorage.clear()
    })

    describe('MockCollectionGateway', () => {
        it('should load empty collections initially', async () => {
            const gateway = new MockCollectionGateway()
            const collections = await gateway.loadCollections()
            expect(collections).toEqual([])
        })

        it('should save and persist collections to localStorage', async () => {
            const gateway = new MockCollectionGateway()
            const testServices: Service[] = [
                {
                    id: 'c1',
                    name: 'Test Collection',
                    directory: '',
                    isAuthenticated: false,
                    endpoints: [],
                    environments: []
                }
            ]
            const saved = await gateway.saveCollections(testServices)
            expect(saved).toEqual(testServices)

            const reloadedGateway = new MockCollectionGateway()
            const reloaded = await reloadedGateway.loadCollections()
            expect(reloaded).toEqual(testServices)
        })
    })

    describe('MockServiceGateway', () => {
        it('should load empty services initially', async () => {
            const gateway = new MockServiceGateway()
            const services = await gateway.loadServices()
            expect(services).toEqual([])
        })

        it('should save and persist services to localStorage', async () => {
            const gateway = new MockServiceGateway()
            const testServices: Service[] = [
                {
                    id: 's1',
                    name: 'Test Service',
                    directory: '/some/path',
                    isAuthenticated: false,
                    endpoints: [],
                    environments: []
                }
            ]
            const saved = await gateway.saveServices(testServices)
            expect(saved).toEqual(testServices)

            const reloadedGateway = new MockServiceGateway()
            const reloaded = await reloadedGateway.loadServices()
            expect(reloaded).toEqual(testServices)
        })

        it('should return default git status', async () => {
            const gateway = new MockServiceGateway()
            const status = await gateway.getGitStatus('/some/path')
            expect(status).toEqual({
                isGit: true,
                branch: 'main',
                hasUncommittedChanges: false,
                hasUnpushedCommits: false
            })
        })

        it('should log or do nothing on sync, pull, push, commit, init', async () => {
            const gateway = new MockServiceGateway()
            await expect(gateway.initGit('/path', 'http://remote')).resolves.not.toThrow()
            await expect(gateway.syncGit('/path')).resolves.not.toThrow()
            await expect(gateway.pullGit('/path')).resolves.not.toThrow()
            await expect(gateway.pushGit('/path')).resolves.not.toThrow()
            await expect(gateway.commitGit('/path', 'commit message')).resolves.not.toThrow()
        })

        it('should import service', async () => {
            const gateway = new MockServiceGateway()
            const service = await gateway.importService('/some/path')
            expect(service.directory).toBe('/some/path')
            expect(service.name).toBe('Imported Service')

            const loaded = await gateway.loadServices()
            expect(loaded).toHaveLength(1)
            expect(loaded[0].directory).toBe('/some/path')
        })

        it('should import curl', async () => {
            const gateway = new MockServiceGateway()
            const testServices: Service[] = [
                {
                    id: 's1',
                    name: 'Test Service',
                    directory: '/some/path',
                    isAuthenticated: false,
                    endpoints: [],
                    environments: []
                }
            ]
            await gateway.saveServices(testServices)

            const service = await gateway.importCurl('s1', 'curl http://url')
            expect(service.id).toBe('s1')
        })

        it('should import swagger and create a service with correct shape', async () => {
            const gateway = new MockServiceGateway()
            const service = await gateway.importSwagger('Swagger Service', '/path/to/spec.json')

            expect(service.name).toBe('Swagger Service')
            expect(service.directory).toBe('/path/to/spec.json')
            expect(service.id).toBeTruthy()
            expect(service.endpoints).toEqual([])
            expect(service.environments).toEqual([])
            expect(service.isAuthenticated).toBe(false)

            // Persisted to localStorage / internal list
            const loaded = await gateway.loadServices()
            expect(loaded).toHaveLength(1)
            expect(loaded[0].name).toBe('Swagger Service')
        })
    })

    describe('MockVersionGateway', () => {
        const config = (url: string) => ({
            method: 'GET',
            url,
            authenticated: false,
            authType: 'none',
            params: [],
            headers: [],
            body: '',
            preflight: null as any,
        })

        it('should return empty versions initially', async () => {
            const gateway = new MockVersionGateway()
            const versions = await gateway.getVersions('s1', 'e1', 50)
            expect(versions).toEqual([])
        })

        it('should auto-increment versions per endpoint and return newest first', async () => {
            const gateway = new MockVersionGateway()
            await gateway.addVersion('s1', 'e1', config('/a'), 50)
            const v2 = await gateway.addVersion('s1', 'e1', config('/b'), 50)
            const other = await gateway.addVersion('s1', 'e2', config('/x'), 50)

            expect(v2.version).toBe(2)
            expect(other.version).toBe(1)

            const versions = await gateway.getVersions('s1', 'e1', 50)
            expect(versions.map(v => v.version)).toEqual([2, 1])
        })

        it('should preserve the config incl. preflight on round trip', async () => {
            const gateway = new MockVersionGateway()
            const cfg: any = {
                method: 'POST',
                url: '/auth',
                authenticated: true,
                authType: 'bearer',
                params: [{ name: 'q', value: '1', enabled: true }],
                headers: [{ name: 'X-Key', value: 'abc', enabled: true }],
                body: '{"a":1}',
                preflight: {
                    method: 'POST',
                    url: 'https://auth.example.com/token',
                    body: '{}',
                    tokenKey: 'access_token',
                    tokenHeader: 'Authorization',
                    enabled: true,
                },
            }
            await gateway.addVersion('s1', 'e1', cfg, 50)
            const versions = await gateway.getVersions('s1', 'e1', 50)
            expect(versions[0]).toMatchObject({
                version: 1,
                config: cfg,
            })
            expect(versions[0].config.preflight.tokenKey).toBe('access_token')
        })

        it('should FIFO prune when exceeding maxVersions', async () => {
            const gateway = new MockVersionGateway()
            for (let i = 0; i < 6; i++) {
                await gateway.addVersion('s1', 'e1', config(`/item-${i}`), 3)
            }
            const versions = await gateway.getVersions('s1', 'e1', 50)
            expect(versions).toHaveLength(3)
            expect(versions.map(v => v.version)).toEqual([6, 5, 4])
        })

        it('should respect the limit on reads', async () => {
            const gateway = new MockVersionGateway()
            for (let i = 0; i < 5; i++) {
                await gateway.addVersion('s1', 'e1', config(`/item-${i}`), 50)
            }
            const limited = await gateway.getVersions('s1', 'e1', 2)
            expect(limited.map(v => v.version)).toEqual([5, 4])
        })

        it('should clear only the targeted endpoint and persist across instances', async () => {
            const gateway = new MockVersionGateway()
            await gateway.addVersion('s1', 'e1', config('/a'), 50)
            await gateway.addVersion('s1', 'e2', config('/b'), 50)

            await gateway.clearVersions('s1', 'e1')

            const reloaded = new MockVersionGateway()
            expect(await reloaded.getVersions('s1', 'e1', 50)).toEqual([])
            expect(await reloaded.getVersions('s1', 'e2', 50)).toHaveLength(1)
        })
    })
})
