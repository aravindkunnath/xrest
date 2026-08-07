<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Plus, Trash2, Eye, EyeOff } from "@lucide/vue";
import { useSecretsStore } from "@/stores/secrets";
import AddSecretDialog from "@/features/dialogs/AddSecretDialog.vue";
import { Button } from "@/components/ui/button";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { Dialogs } from "@wailsio/runtime";
import { toast } from "vue-sonner";

defineProps<{
    embedded?: boolean;
}>();

const secretsStore = useSecretsStore();
const isDialogOpen = ref(false);
const revealedSecrets = ref<Record<string, string>>({});
const isRevealing = ref<Record<string, boolean>>({});

onMounted(() => {
    secretsStore.fetchSecrets();
});

async function handleDelete(key: string) {
    const result = await Dialogs.Question({
        Title: "Delete Secret",
        Message: `Are you sure you want to delete the secret "${key}"? This action cannot be undone.`,
        Buttons: [
            { Label: "Yes", IsDefault: true },
            { Label: "No", IsCancel: true }
        ]
    });
    const confirmed = result === "Yes";

    if (confirmed) {
        try {
            await secretsStore.deleteSecret(key);
            if (revealedSecrets.value[key] !== undefined) {
                const next = { ...revealedSecrets.value };
                delete next[key];
                revealedSecrets.value = next;
            }
            toast.success("Secret deleted successfully");
        } catch (error) {
            toast.error("Failed to delete secret");
            console.error(error);
        }
    }
}

async function toggleReveal(key: string) {
    if (revealedSecrets.value[key] !== undefined) {
        const next = { ...revealedSecrets.value };
        delete next[key];
        revealedSecrets.value = next;
    } else {
        isRevealing.value = { ...isRevealing.value, [key]: true };
        try {
            const value = await secretsStore.getSecret(key);
            revealedSecrets.value = { ...revealedSecrets.value, [key]: value };
        } catch (error) {
            console.error("SecretsView: Failed to get secret:", error);
            toast.error("Failed to reveal secret");
        } finally {
            isRevealing.value = { ...isRevealing.value, [key]: false };
        }
    }
}
</script>

<template>
    <div class="space-y-4 w-full">
        <!-- Top Action Bar: Left Aligned Add Secret Button -->
        <div class="flex items-center justify-start">
            <Button @click="isDialogOpen = true" size="sm" class="flex gap-2 cursor-pointer">
                <Plus class="h-4 w-4" />
                Add Secret
            </Button>
        </div>

        <!-- Loading State -->
        <div
            v-if="secretsStore.isLoading && secretsStore.secrets.length === 0"
            class="py-8 text-sm text-muted-foreground"
        >
            <div class="flex items-center gap-2">
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-primary"></div>
                <span>Loading secrets...</span>
            </div>
        </div>

        <!-- Simple Empty State -->
        <div
            v-else-if="secretsStore.secrets.length === 0"
            class="py-8 text-sm text-muted-foreground"
        >
            No secrets configured
        </div>

        <!-- Simple 2-Column Table -->
        <div v-else class="w-full">
            <Table>
                <TableHeader>
                    <TableRow class="hover:bg-transparent border-b">
                        <TableHead class="w-1/2 text-xs font-semibold">Secret Key</TableHead>
                        <TableHead class="w-1/2 text-xs font-semibold">Value</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <TableRow
                        v-for="key in secretsStore.secrets"
                        :key="key"
                        class="border-b"
                    >
                        <TableCell class="font-mono text-xs font-medium py-3">
                            {{ key }}
                        </TableCell>
                        <TableCell class="py-3">
                            <div class="flex items-center justify-between gap-2">
                                <div class="flex items-center gap-2">
                                    <span
                                        v-if="revealedSecrets[key] !== undefined"
                                        class="font-mono text-xs bg-muted px-2 py-1 rounded"
                                    >
                                        {{ revealedSecrets[key] }}
                                    </span>
                                    <span
                                        v-else
                                        class="text-muted-foreground italic text-xs"
                                        >••••••••••••</span
                                    >
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        class="h-7 w-7 cursor-pointer"
                                        @click="toggleReveal(key)"
                                        :disabled="isRevealing[key]"
                                    >
                                        <EyeOff
                                            v-if="revealedSecrets[key] !== undefined"
                                            class="h-3.5 w-3.5"
                                        />
                                        <Eye v-else class="h-3.5 w-3.5" />
                                    </Button>
                                </div>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10 cursor-pointer"
                                    @click="handleDelete(key)"
                                >
                                    <Trash2 class="h-3.5 w-3.5" />
                                </Button>
                            </div>
                        </TableCell>
                    </TableRow>
                </TableBody>
            </Table>
        </div>

        <AddSecretDialog
            v-model:open="isDialogOpen"
            @success="secretsStore.fetchSecrets"
        />
    </div>
</template>
