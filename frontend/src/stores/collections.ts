import { defineStore } from 'pinia'
import { ref } from 'vue'
import { type Service } from '@/types'
import { toast } from 'vue-sonner'
import { CollectionManager } from '@/domains/collection/manager'
import { AdapterFactory } from '@/infrastructure/adapter-factory'

export const useCollectionsStore = defineStore('collections', () => {
    const collections = ref<Service[]>([])
    const isLoading = ref(false)
    const collectionManager = new CollectionManager(AdapterFactory.getCollectionGateway())

    const loadCollections = async () => {
        isLoading.value = true
        try {
            collections.value = await collectionManager.getAllCollections()
        } catch (error) {
            console.error('Failed to load collections:', error)
            toast.error('Failed to load collections', {
                description: String(error)
            })
        } finally {
            isLoading.value = false
        }
    }

    const saveCollections = async () => {
        try {
            const updated = await collectionManager.saveCollections(collections.value)
            if (updated) {
                collections.value = updated
            }
        } catch (error) {
            console.error('Failed to save collections:', error)
        }
    }

    const addCollection = async (collection: Service) => {
        try {
            const updated = await collectionManager.addCollection(collections.value, collection)
            collections.value = updated
        } catch (error) {
            console.error('Failed to add collection:', error)
            toast.error('Failed to add collection')
        }
    }

    const updateCollection = async (index: number, collection: Service) => {
        try {
            const updated = await collectionManager.updateCollection(collections.value, index, collection)
            collections.value = updated
        } catch (error) {
            console.error('Failed to update collection:', error)
            toast.error('Failed to update collection')
        }
    }

    const deleteCollection = async (index: number) => {
        try {
            const updated = await collectionManager.deleteCollection(collections.value, index)
            collections.value = updated
        } catch (error) {
            console.error('Failed to delete collection:', error)
            toast.error('Failed to delete collection')
        }
    }

    const setSelectedEnvironment = async (collectionId: string, env: string) => {
        const index = collections.value.findIndex(c => c.id === collectionId)
        if (index !== -1) {
            const collection = { ...collections.value[index], selectedEnvironment: env }
            await updateCollection(index, collection)
        }
    }

    return {
        collections,
        isLoading,
        loadCollections,
        saveCollections,
        addCollection,
        updateCollection,
        deleteCollection,
        setSelectedEnvironment
    }
})
