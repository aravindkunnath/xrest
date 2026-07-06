<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { toast } from "vue-sonner";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { useServicesStore } from "@/stores/services";

const props = defineProps<{
    open: boolean;
    serviceId?: string;
}>();

const emit = defineEmits<{
    (e: "update:open", value: boolean): void;
    (e: "import-complete"): void;
}>();

const servicesStore = useServicesStore();
const selectedServiceId = ref(props.serviceId || "");
const curlCommand = ref("");
const isImporting = ref(false);

watch(() => props.serviceId, (newId) => {
    if (newId) selectedServiceId.value = newId;
});

watch(() => props.open, (isOpen) => {
    if (isOpen && props.serviceId) {
        selectedServiceId.value = props.serviceId;
    }
});

const services = computed(() => servicesStore.services);

const handleImportCurl = async () => {
    if (!selectedServiceId.value) {
        toast.error("Please select a service");
        return;
    }
    if (!curlCommand.value.trim()) {
        toast.error("Please enter a cURL command");
        return;
    }

    isImporting.value = true;
    try {
        const updatedService = await servicesStore.importCurl(
            selectedServiceId.value,
            curlCommand.value
        );

        if (updatedService) {
            toast.success("Endpoint Imported", {
                description: "The cURL command has been imported as a new endpoint.",
            });
            emit("import-complete");
            emit("update:open", false);
            // Reset form
            curlCommand.value = "";
        }
    } catch (error) {
        console.error("Failed to import cURL:", error);
        toast.error("Import Failed", {
            description: String(error),
        });
    } finally {
        isImporting.value = false;
    }
};
</script>

<template>
    <Dialog :open="open" @update:open="emit('update:open', $event)">
        <DialogContent class="sm:max-w-[600px]">
            <DialogHeader>
                <DialogTitle>Import from cURL</DialogTitle>
                <DialogDescription>
                    Paste a cURL command and select a service to import it as a new endpoint.
                </DialogDescription>
            </DialogHeader>

            <div class="grid gap-4 py-4">
                <div class="grid gap-2">
                    <Label for="service-select">Target Service</Label>
                    <Select v-model="selectedServiceId">
                        <SelectTrigger id="service-select">
                            <SelectValue placeholder="Select a service..." />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem v-for="service in services" :key="service.id" :value="service.id">
                                {{ service.name }}
                            </SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <div class="grid gap-2">
                    <Label for="curl-command">cURL Command</Label>
                    <Textarea id="curl-command" v-model="curlCommand" placeholder="curl -X POST http://..."
                        class="min-h-[200px] font-mono text-xs" />
                </div>
            </div>

            <DialogFooter>
                <Button variant="outline" @click="emit('update:open', false)">
                    Cancel
                </Button>
                <Button @click="handleImportCurl" :disabled="!selectedServiceId || !curlCommand.trim() || isImporting">
                    {{ isImporting ? "Importing..." : "Import Endpoint" }}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
