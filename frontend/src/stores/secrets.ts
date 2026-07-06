import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSecretsStore = defineStore('secrets', () => {
    const secrets = ref<string[]>([])
    const isLoading = ref(false)
    const error = ref<string | null>(null)

    function getSecretsObject(): Record<string, string> {
        const saved = localStorage.getItem('xrest_secrets')
        return saved ? JSON.parse(saved) : {}
    }

    async function fetchSecrets() {
        isLoading.value = true
        error.value = null
        try {
            const secretsObj = getSecretsObject()
            secrets.value = Object.keys(secretsObj)
        } catch (e) {
            error.value = String(e)
            console.error('Failed to fetch secrets:', e)
        } finally {
            isLoading.value = false
        }
    }

    async function getSecret(key: string): Promise<string> {
        const secretsObj = getSecretsObject()
        return secretsObj[key] || ''
    }

    async function addSecret(key: string, value: string) {
        isLoading.value = true
        error.value = null
        try {
            const secretsObj = getSecretsObject()
            secretsObj[key] = value
            localStorage.setItem('xrest_secrets', JSON.stringify(secretsObj))
            secrets.value = Object.keys(secretsObj)
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
            const secretsObj = getSecretsObject()
            delete secretsObj[key]
            localStorage.setItem('xrest_secrets', JSON.stringify(secretsObj))
            secrets.value = Object.keys(secretsObj)
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
