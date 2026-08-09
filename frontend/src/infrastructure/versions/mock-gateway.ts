import type { IVersionGateway } from "@/domains/versions/ports"
import type { RequestConfig, EndpointVersion } from "@/types"

const STORAGE_KEY = 'xrest_versions'

// Each entry: endpointId -> EndpointVersion[] stored newest-first.
type VersionMap = Record<string, EndpointVersion[]>

export class MockVersionGateway implements IVersionGateway {
    private readAll(): VersionMap {
        const saved = localStorage.getItem(STORAGE_KEY)
        return saved ? (JSON.parse(saved) as VersionMap) : {}
    }

    private writeAll(map: VersionMap): void {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(map))
    }

    async getVersions(_serviceId: string, endpointId: string, limit: number): Promise<EndpointVersion[]> {
        const all = this.readAll()
        const entries = all[endpointId] || []
        return entries.slice(0, limit)
    }

    async addVersion(
        _serviceId: string,
        endpointId: string,
        config: RequestConfig,
        maxVersions: number,
    ): Promise<EndpointVersion> {
        const all = this.readAll()
        const entries = all[endpointId] || []
        const maxVersion = entries.reduce((max, v) => Math.max(max, v.version), 0)
        const newVersion: EndpointVersion = {
            version: maxVersion + 1,
            config: JSON.parse(JSON.stringify(config)),
            lastUpdated: Math.floor(Date.now() / 1000),
        }
        entries.unshift(newVersion)
        if (entries.length > maxVersions) {
            entries.length = maxVersions
        }
        all[endpointId] = entries
        this.writeAll(all)
        return newVersion
    }

    async clearVersions(_serviceId: string, endpointId: string): Promise<void> {
        const all = this.readAll()
        delete all[endpointId]
        this.writeAll(all)
    }
}