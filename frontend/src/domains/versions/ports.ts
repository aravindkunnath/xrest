import type { RequestConfig, EndpointVersion } from "@/types"

export interface IVersionGateway {
    getVersions(serviceId: string, endpointId: string, limit: number): Promise<EndpointVersion[]>
    addVersion(
        serviceId: string,
        endpointId: string,
        config: RequestConfig,
        maxVersions: number,
    ): Promise<EndpointVersion>
    clearVersions(serviceId: string, endpointId: string): Promise<void>
}
