import { defineStore } from 'pinia'
import { reactive } from 'vue'
import { AdapterFactory } from "@/infrastructure/adapter-factory"
import type { RequestConfig, EndpointVersion } from "@/types"

interface VersionCacheEntry {
    entries: EndpointVersion[]
    isLoading: boolean
    count: number
    loaded: boolean
}

export const useVersionsStore = defineStore('versions', () => {
    const cache = reactive<Record<string, VersionCacheEntry>>({})
    const gateway = AdapterFactory.getVersionGateway()

    function ensure(endpointId: string): VersionCacheEntry {
        if (!cache[endpointId]) {
            cache[endpointId] = { entries: [], isLoading: false, count: 0, loaded: false }
        }
        return cache[endpointId]
    }

    async function loadVersions(serviceId: string, endpointId: string, limit: number) {
        const entry = ensure(endpointId)
        entry.isLoading = true
        try {
            const versions = await gateway.getVersions(serviceId, endpointId, limit)
            entry.entries = versions
            entry.count = versions.length
            entry.loaded = true
        } catch (err) {
            console.error('Failed to load versions:', err)
        } finally {
            entry.isLoading = false
        }
    }

    async function addVersion(
        serviceId: string,
        endpointId: string,
        config: RequestConfig,
        maxVersions: number,
    ): Promise<EndpointVersion> {
        const created = await gateway.addVersion(serviceId, endpointId, config, maxVersions)
        const entry = ensure(endpointId)
        entry.entries = [created, ...entry.entries].slice(0, maxVersions)
        entry.count = entry.entries.length
        entry.loaded = true
        return created
    }

    function getCount(endpointId: string): number {
        return cache[endpointId]?.count ?? 0
    }

    function getEntries(endpointId: string): EndpointVersion[] {
        return cache[endpointId]?.entries ?? []
    }

    function isLoading(endpointId: string): boolean {
        return cache[endpointId]?.isLoading ?? false
    }

    async function clearVersions(serviceId: string, endpointId: string) {
        await gateway.clearVersions(serviceId, endpointId)
        const entry = ensure(endpointId)
        entry.entries = []
        entry.count = 0
        entry.loaded = false
    }

    return {
        cache,
        loadVersions,
        addVersion,
        getCount,
        getEntries,
        isLoading,
        clearVersions,
    }
})