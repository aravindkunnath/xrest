import { defineStore } from 'pinia'
import { ref } from 'vue'
import { GetSecrets, GetSecret, AddSecret, DeleteSecret } from '../../bindings/xrest/cmd/wails/secretsgateway'

export const useSecretsStore = defineStore('secrets', () => {
    const secrets = ref<string[]>([])
    const isLoading = ref(false)
    const error = ref<string | null>(null)

    async function fetchSecrets() {
        isLoading.value = true
        error.value = null
        try {
            secrets.value = (await GetSecrets()) || []
        } catch (e) {
            error.value = String(e)
            console.error('Failed to fetch secrets:', e)
        } finally {
            isLoading.value = false
        }
    }

    async function getSecret(key: string): Promise<string> {
        try {
            return await GetSecret(key)
        } catch (e) {
            console.error(`Failed to get secret for key ${key}:`, e)
            return ''
        }
    }

    async function addSecret(key: string, value: string) {
        isLoading.value = true
        error.value = null
        try {
            secrets.value = (await AddSecret(key, value)) || []
        } catch (e) {
            error.value = String(e)
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
            secrets.value = (await DeleteSecret(key)) || []
        } catch (e) {
            error.value = String(e)
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
        getSecret,
        addSecret,
        deleteSecret
    }
})
