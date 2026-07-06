import type { Service } from '@/types'
import type { IServiceGateway } from '@/domains/service/ports'

export class MockServiceGateway implements IServiceGateway {
    private services: Service[] = []

    constructor() {
        // Load from local storage if available to persist across reloads in browser
        const stored = localStorage.getItem('mock_services')
        if (stored) {
            this.services = JSON.parse(stored)
        }
    }

    async loadServices(): Promise<Service[]> {
        return [...this.services]
    }

    async saveServices(services: Service[], _commitMessage?: string): Promise<Service[]> {
        this.services = services
        localStorage.setItem('mock_services', JSON.stringify(services))
        return services
    }

    async getGitStatus(_directory: string): Promise<any> {
        return {
            isGit: true,
            branch: 'main',
            hasUncommittedChanges: false,
            hasUnpushedCommits: false
        }
    }

    async initGit(directory: string, remoteUrl?: string): Promise<void> {
        console.log('Mock init git:', directory, remoteUrl)
    }

    async syncGit(directory: string): Promise<void> {
        console.log('Mock sync git:', directory)
    }

    async pullGit(directory: string): Promise<void> {
        console.log('Mock pull git:', directory)
    }

    async pushGit(directory: string): Promise<void> {
        console.log('Mock push git:', directory)
    }

    async commitGit(directory: string, message: string): Promise<void> {
        console.log('Mock commit git:', directory, message)
    }

    async importService(directory: string): Promise<Service> {
        const id = `s-${Date.now()}`
        const service: Service = {
            id,
            name: 'Imported Service',
            directory,
            isAuthenticated: false,
            endpoints: [],
            environments: []
        }
        this.services.push(service)
        this.saveServices(this.services)
        return service
    }

    async importCurl(serviceId: string, curlCommand: string): Promise<Service> {
        console.log('Mock import curl:', serviceId, curlCommand)
        const service = this.services.find(s => s.id === serviceId)
        if (!service) throw new Error('Service not found')
        return service
    }
}
