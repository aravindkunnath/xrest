import { invoke } from '@tauri-apps/api/core'
import type { Service } from '@/types'
import type { IServiceGateway } from '@/domains/service/ports'

export class TauriServiceGateway implements IServiceGateway {
    async loadServices(): Promise<Service[]> {
        return invoke<Service[]>('get_services')
    }

    async saveServices(services: Service[], commitMessage?: string): Promise<Service[]> {
        return invoke<Service[]>('save_services', { services, commitMessage })
    }

    async getGitStatus(directory: string): Promise<any> {
        return invoke<any>('get_git_status', { directory })
    }

    async initGit(directory: string, remoteUrl?: string): Promise<void> {
        return invoke('git_init', { directory, remoteUrl })
    }

    async syncGit(directory: string): Promise<void> {
        return invoke('git_sync', { directory })
    }

    async pullGit(directory: string): Promise<void> {
        return invoke('git_pull', { directory })
    }

    async pushGit(directory: string): Promise<void> {
        return invoke('git_push', { directory })
    }

    async commitGit(directory: string, message: string): Promise<void> {
        return invoke('git_commit', { directory, message })
    }

    async importService(directory: string): Promise<Service> {
        return invoke('import_service', { directory })
    }

    async importCurl(serviceId: string, curlCommand: string): Promise<Service> {
        return invoke('import_curl', { serviceId, curlCommand })
    }
}
