<script setup lang="ts">
import { ref } from 'vue'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useSecretsStore } from '@/stores/secrets'

defineProps<{
    open: boolean
}>()

const emit = defineEmits<{
    (e: 'update:open', value: boolean): void
    (e: 'success'): void
}>()

const secretsStore = useSecretsStore()
const keyName = ref('')
const secretValue = ref('')
const isSubmitting = ref(false)

async function handleSubmit() {
    if (!keyName.value || !secretValue.value) return

    isSubmitting.value = true
    try {
        await secretsStore.addSecret(keyName.value, secretValue.value)
        keyName.value = ''
        secretValue.value = ''
        emit('update:open', false)
        emit('success')
    } catch (error) {
        console.error('Failed to add secret:', error)
    } finally {
        isSubmitting.value = false
    }
}
</script>

<template>
    <Dialog :open="open" @update:open="emit('update:open', $event)">
        <DialogContent class="sm:max-w-[425px]">
            <DialogHeader>
                <DialogTitle>Add New Secret</DialogTitle>
                <DialogDescription>
                    Enter a key name and its secret value. This will be stored securely in your macOS Keychain.
                </DialogDescription>
            </DialogHeader>
            <div class="grid gap-4 py-4">
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="key" class="text-right">Key Name</Label>
                    <Input id="key" v-model="keyName" placeholder="Secret Name (e.g. OPENAI_API_KEY)" class="col-span-3"
                        @keyup.enter="handleSubmit" />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="value" class="text-right">Secret Value</Label>
                    <Input id="value" v-model="secretValue" type="password" placeholder="Secret Value"
                        class="col-span-3" @keyup.enter="handleSubmit" />
                </div>
            </div>
            <DialogFooter>
                <Button variant="outline" @click="emit('update:open', false)">Cancel</Button>
                <Button @click="handleSubmit" :disabled="isSubmitting || !keyName || !secretValue">
                    {{ isSubmitting ? 'Saving...' : 'Save Secret' }}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
