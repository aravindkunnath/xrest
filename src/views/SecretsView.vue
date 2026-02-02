<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus, Trash2, Shield, Eye, EyeOff } from 'lucide-vue-next'
import { useSecretsStore } from '@/stores/secrets'
import AddSecretDialog from '@/components/AddSecretDialog.vue'
import { Button } from '@/components/ui/button'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { invoke } from '@tauri-apps/api/core'
import { ask } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue-sonner'

const secretsStore = useSecretsStore()
const isDialogOpen = ref(false)
const revealedSecrets = ref<Record<string, string>>({})
const isRevealing = ref<Record<string, boolean>>({})

onMounted(() => {
    secretsStore.fetchSecrets()
})

async function handleDelete(key: string) {
    const confirmed = await ask(
        `Are you sure you want to delete the secret "${key}"? This action cannot be undone.`,
        {
            title: 'Delete Secret',
            kind: 'warning',
        }
    )

    if (confirmed) {
        try {
            await secretsStore.deleteSecret(key)
            if (revealedSecrets.value[key] !== undefined) {
                const next = { ...revealedSecrets.value }
                delete next[key]
                revealedSecrets.value = next
            }
            toast.success('Secret deleted successfully')
        } catch (error) {
            toast.error('Failed to delete secret')
            console.error(error)
        }
    }
}

async function toggleReveal(key: string) {
    if (revealedSecrets.value[key] !== undefined) {
        const next = { ...revealedSecrets.value }
        delete next[key]
        revealedSecrets.value = next
    } else {
        isRevealing.value = { ...isRevealing.value, [key]: true }
        try {
            const value = await invoke<string>('get_secret', { key })
            revealedSecrets.value = { ...revealedSecrets.value, [key]: value }
        } catch (error) {
            console.error('SecretsView: Failed to get secret:', error)
            toast.error('Failed to reveal secret')
        } finally {
            isRevealing.value = { ...isRevealing.value, [key]: false }
        }
    }
}
</script>

<template>
    <div class="p-6 space-y-6">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="text-3xl font-bold tracking-tight">Secrets Management</h1>
                <p class="text-muted-foreground mt-1">
                    Manage your sensitive keys and credentials securely stored in the macOS Keychain.
                </p>
            </div>
            <Button @click="isDialogOpen = true" class="flex gap-2">
                <Plus class="h-4 w-4" />
                Add Secret
            </Button>
        </div>

        <div class="rounded-xl border bg-card text-card-foreground shadow">
            <div class="p-6 flex flex-col space-y-1.5">
                <div class="font-semibold leading-none tracking-tight flex items-center gap-2">
                    <Shield class="h-5 w-5 text-primary" />
                    Workspace Secrets
                </div>
                <p class="text-sm text-muted-foreground">
                    These secrets can be used as variables in your requests using
                    <code v-pre>{{secret.KEY_NAME}}</code> syntax.
                </p>
            </div>
            <div class="p-6 pt-0">
                <div v-if="secretsStore.isLoading && secretsStore.secrets.length === 0" class="py-20 text-center">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
                    <p class="text-muted-foreground">Loading secrets...</p>
                </div>

                <div v-else-if="secretsStore.secrets.length === 0"
                    class="py-20 text-center border-2 border-dashed rounded-lg">
                    <Shield class="h-12 w-12 text-muted-foreground mx-auto mb-4 opacity-20" />
                    <h3 class="text-lg font-medium">No secrets found</h3>
                    <p class="text-sm text-muted-foreground max-w-sm mx-auto mt-2">
                        You haven't added any secrets yet. Secrets are stored securely and are not synced to git.
                    </p>
                    <Button variant="outline" @click="isDialogOpen = true" class="mt-4 gap-2">
                        <Plus class="h-4 w-4" />
                        Add your first secret
                    </Button>
                </div>

                <div v-else class="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Secret Key</TableHead>
                                <TableHead>Value</TableHead>
                                <TableHead class="text-right">Actions</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <TableRow v-for="key in secretsStore.secrets" :key="key">
                                <TableCell class="font-mono font-medium">{{ key }}</TableCell>
                                <TableCell>
                                    <div class="flex items-center gap-2">
                                        <span v-if="revealedSecrets[key] !== undefined"
                                            class="font-mono text-xs bg-muted px-2 py-1 rounded">
                                            {{ revealedSecrets[key] }}
                                        </span>
                                        <span v-else class="text-muted-foreground italic">••••••••••••</span>
                                        <Button variant="ghost" size="icon" class="h-8 w-8" @click="toggleReveal(key)"
                                            :disabled="isRevealing[key]">
                                            <EyeOff v-if="revealedSecrets[key] !== undefined" class="h-4 w-4" />
                                            <Eye v-else class="h-4 w-4" />
                                        </Button>
                                    </div>
                                </TableCell>
                                <TableCell class="text-right">
                                    <Button variant="ghost" size="icon"
                                        class="h-8 w-8 text-destructive hover:text-destructive hover:bg-destructive/10"
                                        @click="handleDelete(key)">
                                        <Trash2 class="h-4 w-4" />
                                    </Button>
                                </TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </div>
            </div>
        </div>

        <AddSecretDialog v-model:open="isDialogOpen" @success="secretsStore.fetchSecrets" />
    </div>
</template>
