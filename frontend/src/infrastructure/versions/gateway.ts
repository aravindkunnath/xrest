import { VersionGateway as WailsVersionGateway } from '../../../bindings/xrest/cmd/wails'
import type { IVersionGateway } from "@/domains/versions/ports"
import type { RequestConfig, EndpointVersion } from "@/types"

export class VersionGateway implements IVersionGateway {
    async getVersions(serviceId: string, endpointId: string, limit: number): Promise<EndpointVersion[]> {
        const result = await WailsVersionGateway.GetEndpointVersions(serviceId, endpointId, limit)
        return (result as unknown as EndpointVersion[]) || []
    }

    async addVersion(
        serviceId: string,
        endpointId: string,
        config: RequestConfig,
        maxVersions: number,
    ): Promise<EndpointVersion> {
        const result = await WailsVersionGateway.AddEndpointVersion(serviceId, endpointId, config as any, maxVersions)
        return result as unknown as EndpointVersion
    }

    async clearVersions(serviceId: string, endpointId: string): Promise<void> {
        await WailsVersionGateway.ClearEndpointVersions(serviceId, endpointId)
    }
}