import { defineStore } from 'pinia'
import { ref } from 'vue'
import { type Service } from '@/types'
import { toast } from 'vue-sonner'
import { ServiceManager } from '@/domains/service/manager'
import { AdapterFactory } from '@/infrastructure/adapter-factory'

export const useServicesStore = defineStore('services', () => {
    const services = ref<Service[]>([])
    const isLoading = ref(false)
    const serviceManager = new ServiceManager(AdapterFactory.getServiceGateway())

    const loadServices = async () => {
        isLoading.value = true
        try {
            services.value = await serviceManager.getAllServices()
        } catch (error) {
            console.error('Failed to load services:', error)
            toast.error('Failed to load services', {
                description: String(error)
            })
        } finally {
            isLoading.value = false
        }
    }

    const saveServices = async (commitMessage?: string) => {
        try {
            const updated = await serviceManager.saveServices(services.value, commitMessage)
            if (updated) {
                services.value = updated
            }
        } catch (error) {
            console.error('Failed to save services:', error)
        }
    }

    const addService = async (service: Service, commitMessage?: string) => {
        try {
            const updated = await serviceManager.addService(services.value, service, commitMessage)
            services.value = updated
        } catch (error) {
            console.error('Failed to add service:', error)
            toast.error('Failed to add service')
        }
    }

    const updateService = async (index: number, service: Service, commitMessage?: string) => {
        try {
            const updated = await serviceManager.updateService(services.value, index, service, commitMessage)
            services.value = updated
        } catch (error) {
            console.error('Failed to update service:', error)
            toast.error('Failed to update service')
        }
    }

    const deleteService = async (index: number) => {
        try {
            const updated = await serviceManager.deleteService(services.value, index)
            services.value = updated
        } catch (error) {
            console.error('Failed to delete service:', error)
            toast.error('Failed to delete service')
        }
    }

    const setSelectedEnvironment = async (serviceId: string, env: string) => {
        const index = services.value.findIndex(s => s.id === serviceId)
        if (index !== -1) {
            const service = { ...services.value[index], selectedEnvironment: env }
            await updateService(index, service)
        }
    }

    const getGitStatus = async (directory: string) => {
        try {
            return await serviceManager.getGitStatus(directory)
        } catch (error) {
            console.error('Failed to get git status:', error)
            return null
        }
    }

    const initGit = async (directory: string, remoteUrl?: string) => {
        try {
            await serviceManager.initGit(directory, remoteUrl)
            toast.success('Git initialized successfully')
        } catch (error) {
            toast.error('Failed to initialize git', {
                description: String(error)
            })
        }
    }

    const syncGit = async (directory: string) => {
        try {
            await serviceManager.syncGit(directory)
            toast.success('Git sync completed')
        } catch (error) {
            toast.error('Failed to sync git', {
                description: String(error)
            })
        }
    }

    const pullGit = async (directory: string) => {
        try {
            await serviceManager.pullGit(directory)
            toast.success('Git pull completed')
        } catch (error) {
            toast.error('Failed to pull from remote', {
                description: String(error)
            })
        }
    }

    const pushGit = async (directory: string) => {
        try {
            await serviceManager.pushGit(directory)
            toast.success('Git push completed')
        } catch (error) {
            toast.error('Failed to push to remote', {
                description: String(error)
            })
        }
    }

    const commitGit = async (directory: string, message: string) => {
        try {
            await serviceManager.commitGit(directory, message)
            toast.success('Changes committed')
        } catch (error) {
            toast.error('Failed to commit changes', {
                description: String(error)
            })
        }
    }

    const importService = async (directory: string): Promise<Service | null> => {
        try {
            const service = await serviceManager.importService(directory)
            if (service) {
                await loadServices()
                return service
            }
        } catch (error) {
            console.error('Failed to import service:', error)
            throw error
        }
        return null
    }

    const importCurl = async (serviceId: string, curlCommand: string): Promise<Service | null> => {
        try {
            const updatedService = await serviceManager.importCurl(serviceId, curlCommand)
            if (updatedService) {
                const index = services.value.findIndex(s => s.id === serviceId)
                if (index !== -1) {
                    services.value[index] = updatedService
                } else {
                    services.value.push(updatedService)
                }
                return updatedService
            }
        } catch (error) {
            console.error('Failed to import curl:', error)
            toast.error('Import Failed', {
                description: String(error)
            })
        }
        return null
    }

    return {
        services,
        isLoading,
        loadServices,
        saveServices,
        addService,
        updateService,
        deleteService,
        setSelectedEnvironment,
        getGitStatus,
        initGit,
        syncGit,
        pullGit,
        pushGit,
        commitGit,
        importService,
        importCurl
    }
})
