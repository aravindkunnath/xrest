import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useVersionsStore } from "@/stores/versions"

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

describe('Versions Store', () => {
    beforeEach(() => {
        setActivePinia(createPinia())
        localStorage.clear()
    })

    it('should load versions and populate the cache count', async () => {
        const store = useVersionsStore()
        for (let i = 0; i < 3; i++) {
            await store.addVersion('s1', 'e1', config(`/item-${i}`), 50)
        }

        // Reset the cache so load goes through the gateway path.
        delete store.cache.e1
        await store.loadVersions('s1', 'e1', 50)

        expect(store.getCount('e1')).toBe(3)
        const entries = store.getEntries('e1')
        expect(entries.map(v => v.version)).toEqual([3, 2, 1])
    })

    it('should return 0 count for unknown endpoints', async () => {
        const store = useVersionsStore()
        expect(store.getCount('nope')).toBe(0)
        expect(store.getEntries('nope')).toEqual([])
        expect(store.isLoading('nope')).toBe(false)
    })

    it('should add a version, return it, and bump the count', async () => {
        const store = useVersionsStore()

        const v1 = await store.addVersion('s1', 'e1', config('/a'), 50)
        expect(v1.version).toBe(1)
        expect(v1.config.url).toBe('/a')
        expect(store.getCount('e1')).toBe(1)

        const v2 = await store.addVersion('s1', 'e1', config('/b'), 50)
        expect(v2.version).toBe(2)
        expect(store.getCount('e1')).toBe(2)
    })

    it('should FIFO-trim the in-memory cache on add beyond maxVersions', async () => {
        const store = useVersionsStore()
        for (let i = 0; i < 6; i++) {
            await store.addVersion('s1', 'e1', config(`/item-${i}`), 3)
        }
        expect(store.getCount('e1')).toBe(3)
        expect(store.getEntries('e1').map(v => v.version)).toEqual([6, 5, 4])
    })

    it('should clear versions for an endpoint and reset the badge count', async () => {
        const store = useVersionsStore()
        await store.addVersion('s1', 'e1', config('/a'), 50)
        await store.addVersion('s1', 'e2', config('/b'), 50)

        await store.clearVersions('s1', 'e1')

        expect(store.getCount('e1')).toBe(0)
        expect(store.getEntries('e1')).toEqual([])
        expect(store.getCount('e2')).toBe(1)
    })
})