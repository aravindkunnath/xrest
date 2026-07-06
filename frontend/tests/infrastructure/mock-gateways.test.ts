import { describe, it, expect, beforeEach } from 'vitest'
import { MockCollectionGateway } from '@/infrastructure/collection/mock-gateway'
import { MockServiceGateway } from '@/infrastructure/service/mock-gateway'
import type { Service } from '@/types'

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
    })
})
