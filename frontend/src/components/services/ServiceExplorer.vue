<script setup lang="ts">
import ServiceTree from "@/components/ServiceTree.vue";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import { useDialogState } from "@/composables/useDialogState";
import { useServicesStore } from "@/stores/services";
import { Dialogs } from "@wailsio/runtime";
import { Download, Plus, Search, X } from "@lucide/vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { toast } from "vue-sonner";

const emit = defineEmits<{
    (e: "select-endpoint", endpoint: any): void;
    (e: "select-service-settings", service: any): void;
    (e: "env-change", serviceId: string, env: string): void;
    (e: "import-curl", service: any): void;
}>();

const servicesStore = useServicesStore();
const {
    isServiceDialogOpen,
    isEndpointDialogOpen,
    isSwaggerDialogOpen,
    isCurlDialogOpen,
} = useDialogState();

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const isImportPopoverOpen = ref(false);
const isAddPopoverOpen = ref(false);

// Handle global keydown for search shortcut
const handleGlobalKeydown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        searchInput.value?.focus();
    }
};

onMounted(() => {
    window.addEventListener("keydown", handleGlobalKeydown);
});

onUnmounted(() => {
    window.removeEventListener("keydown", handleGlobalKeydown);
});

const filteredServices = computed(() => {
    if (!searchQuery.value) return servicesStore.services;

    const query = searchQuery.value.toLowerCase();
    return servicesStore.services
        .map((service) => {
            // Check if service name matches
            const serviceMatches = service.name.toLowerCase().includes(query);

            // Filter endpoints that match
            const matchingEndpoints = service.endpoints.filter(
                (endpoint) =>
                    endpoint.name.toLowerCase().includes(query) ||
                    endpoint.url.toLowerCase().includes(query) ||
                    endpoint.method.toLowerCase().includes(query),
            );

            if (serviceMatches) {
                return service;
            }

            if (matchingEndpoints.length > 0) {
                return {
                    ...service,
                    endpoints: matchingEndpoints,
                    isOpen: true, // Force open if we found matches inside
                };
            }

            return null;
        })
        .filter((s): s is any => s !== null);
});

// Handlers
const handleImportService = async () => {
    const directory = await Dialogs.OpenFile({
        CanChooseDirectories: true,
        CanChooseFiles: false,
        AllowsMultipleSelection: false,
        Title: "Select Service Directory",
    });
    if (!directory) return;

    const dirPath = Array.isArray(directory) ? directory[0] : directory;

    try {
        const service: any = await servicesStore.importService(dirPath);
        if (service) {
            toast.success("Service Imported", {
                description: `Service "${service.name}" has been imported successfully.`,
            });
        }
    } catch (error) {
        console.error("Failed to import service:", error);
        toast.error("Import Failed", {
            description: String(error),
        });
    }
};
</script>

<template>
    <div class="flex flex-col h-full bg-muted/5 border-r">
        <!-- Header / Search -->
        <div class="p-3 border-b flex items-center justify-between">
            <div class="relative flex-1 mr-2">
                <Search
                    class="absolute left-2 top-2.5 h-3.5 w-3.5 text-muted-foreground"
                />
                <input
                    ref="searchInput"
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search services... (Cmd+K)"
                    class="w-full bg-background border rounded-md py-1.5 pl-7 pr-8 focus:outline-none focus:ring-1 focus:ring-primary"
                />
                <button
                    v-if="searchQuery"
                    @click="searchQuery = ''"
                    class="absolute right-2 top-2 p-0.5 hover:bg-muted rounded-sm transition-colors"
                >
                    <X class="h-3 w-3 text-muted-foreground" />
                </button>
            </div>

            <div class="flex items-center gap-1">
                <Popover v-model:open="isImportPopoverOpen">
                    <PopoverTrigger as-child>
                        <button
                            class="p-1.5 hover:bg-muted rounded-md transition-colors text-muted-foreground"
                            title="Import options"
                        >
                            <Download class="h-4 w-4" />
                        </button>
                    </PopoverTrigger>
                    <PopoverContent class="w-48 p-1" align="start">
                        <div class="flex flex-col">
                            <button
                                @click="
                                    isImportPopoverOpen = false;
                                    handleImportService();
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>From Directory</span>
                            </button>
                            <button
                                @click="
                                    isImportPopoverOpen = false;
                                    isSwaggerDialogOpen = true;
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>Swagger / OpenAPI</span>
                            </button>
                            <button
                                @click="
                                    isImportPopoverOpen = false;
                                    isCurlDialogOpen = true;
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>cURL Command</span>
                            </button>
                        </div>
                    </PopoverContent>
                </Popover>

                <Popover v-model:open="isAddPopoverOpen">
                    <PopoverTrigger as-child>
                        <button
                            class="p-1.5 hover:bg-muted rounded-md transition-colors text-muted-foreground"
                        >
                            <Plus class="h-4 w-4" />
                        </button>
                    </PopoverTrigger>
                    <PopoverContent class="w-48 p-1" align="end">
                        <div class="flex flex-col">
                            <button
                                @click="
                                    isAddPopoverOpen = false;
                                    isServiceDialogOpen = true;
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>Add New Service</span>
                            </button>
                            <button
                                @click="
                                    isAddPopoverOpen = false;
                                    isEndpointDialogOpen = true;
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>Add New Endpoint</span>
                            </button>
                            <button
                                @click="
                                    isAddPopoverOpen = false;
                                    handleImportService();
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>Import from Directory</span>
                            </button>
                            <button
                                @click="
                                    isAddPopoverOpen = false;
                                    isSwaggerDialogOpen = true;
                                "
                                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
                            >
                                <span>Import from Swagger</span>
                            </button>
                        </div>
                    </PopoverContent>
                </Popover>
            </div>
        </div>

        <!-- Tree -->
        <div class="flex-1 overflow-auto p-2">
            <ServiceTree
                :services="filteredServices"
                @select-endpoint="emit('select-endpoint', $event)"
                @select-service-settings="
                    emit('select-service-settings', $event)
                "
                @env-change="(id, env) => emit('env-change', id, env)"
                @import-curl="emit('import-curl', $event)"
            />
        </div>
    </div>
</template>
