import type { Service } from '@/types'
import type { IServiceGateway } from './ports'

export class ServiceManager {
    constructor(private gateway: IServiceGateway) { }

    async getAllServices(): Promise<Service[]> {
        return this.gateway.loadServices()
    }

    async saveServices(services: Service[], commitMessage?: string): Promise<Service[]> {
        return this.gateway.saveServices(services, commitMessage)
    }

    async addService(services: Service[], service: Service, commitMessage?: string): Promise<Service[]> {
        const newServices = [...services, service]
        return this.saveServices(newServices, commitMessage)
    }

    async updateService(services: Service[], index: number, service: Service, commitMessage?: string): Promise<Service[]> {
        const newServices = [...services]
        newServices[index] = service
        return this.saveServices(newServices, commitMessage)
    }

    async deleteService(services: Service[], index: number): Promise<Service[]> {
        const newServices = [...services]
        newServices.splice(index, 1)
        return this.saveServices(newServices)
    }

    async getGitStatus(directory: string): Promise<any> {
        return this.gateway.getGitStatus(directory)
    }

    async initGit(directory: string, remoteUrl?: string): Promise<void> {
        return this.gateway.initGit(directory, remoteUrl)
    }

    async syncGit(directory: string): Promise<void> {
        return this.gateway.syncGit(directory)
    }

    async pullGit(directory: string): Promise<void> {
        return this.gateway.pullGit(directory)
    }

    async pushGit(directory: string): Promise<void> {
        return this.gateway.pushGit(directory)
    }

    async commitGit(directory: string, message: string): Promise<void> {
        return this.gateway.commitGit(directory, message)
    }

    async importService(directory: string): Promise<Service> {
        return this.gateway.importService(directory)
    }

    async importCurl(serviceId: string, curlCommand: string): Promise<Service> {
        return this.gateway.importCurl(serviceId, curlCommand)
    }
}
