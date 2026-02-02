import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSecretsStore = defineStore('secrets', () => {
    const secrets = ref<string[]>([])
    const isLoading = ref(false)
    const error = ref<string | null>(null)

    async function fetchSecrets() {
        isLoading.value = true
        error.value = null
        try {
            secrets.value = await invoke<string[]>('get_secrets')
        } catch (e) {
            error.value = e as string
            console.error('Failed to fetch secrets:', e)
        } finally {
            isLoading.value = false
        }
    }

    async function addSecret(key: string, value: string) {
        isLoading.value = true
        error.value = null
        try {
            secrets.value = await invoke<string[]>('add_secret', { key, value })
        } catch (e) {
            error.value = e as string
            console.error('Failed to add secret:', e)
            throw e
        } finally {
            isLoading.value = false
        }
    }

    async function deleteSecret(key: string) {
        isLoading.value = true
        error.value = null
        try {
            secrets.value = await invoke<string[]>('delete_secret', { key })
        } catch (e) {
            error.value = e as string
            console.error('Failed to delete secret:', e)
        } finally {
            isLoading.value = false
        }
    }

    return {
        secrets,
        isLoading,
        error,
        fetchSecrets,
        addSecret,
        deleteSecret
    }
})
