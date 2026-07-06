import type { Service } from '@/types'

export interface IServiceGateway {
    loadServices(): Promise<Service[]>;
    saveServices(services: Service[], commitMessage?: string): Promise<Service[]>;
    getGitStatus(directory: string): Promise<any>;
    initGit(directory: string, remoteUrl?: string): Promise<void>;
    syncGit(directory: string): Promise<void>;
    pullGit(directory: string): Promise<void>;
    pushGit(directory: string): Promise<void>;
    commitGit(directory: string, message: string): Promise<void>;
    importService(directory: string): Promise<Service>;
    importCurl(serviceId: string, curlCommand: string): Promise<Service>;
}
