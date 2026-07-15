<script setup lang="ts">
import ServiceTree from "@/components/ServiceTree.vue";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import { useDialogState } from "@/composables/useDialogState";
import { useServicesStore } from "@/stores/services";
import { useCollectionsStore } from "@/stores/collections";
import { Dialogs } from "@wailsio/runtime";
import { Download, Plus, Search, X, ChevronDown, ChevronRight, Layers, Folder, PlusCircle } from "@lucide/vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { toast } from "vue-sonner";

const emit = defineEmits<{
    (e: "select-endpoint", endpoint: any): void;
    (e: "select-service-settings", service: any): void;
    (e: "env-change", serviceId: string, env: string): void;
    (e: "import-curl", service: any): void;
    (e: "endpoint-context", event: MouseEvent, endpoint: any): void;
}>();

const servicesStore = useServicesStore();
const collectionsStore = useCollectionsStore();
const {
    isServiceDialogOpen,
    isEndpointDialogOpen,
    isSwaggerDialogOpen,
    isCurlDialogOpen,
    isCollectionDialogOpen,
    isCollectionEndpointDialogOpen,
} = useDialogState();

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

// Collapsible states
const isServicesExpanded = ref(true);
const isScratchpadExpanded = ref(true);

const isImportPopoverOpen = ref(false);
const isAddPopoverOpen = ref(false);
const isCollectionPopoverOpen = ref(false);

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
            const serviceMatches = service.name.toLowerCase().includes(query);
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
                    isOpen: true,
                };
            }

            return null;
        })
        .filter((s): s is any => s !== null);
});

const filteredCollections = computed(() => {
    if (!searchQuery.value) return collectionsStore.collections;

    const query = searchQuery.value.toLowerCase();
    return collectionsStore.collections
        .map((collection) => {
            const collectionMatches = collection.name.toLowerCase().includes(query);
            const matchingEndpoints = collection.endpoints.filter(
                (endpoint) =>
                    endpoint.name.toLowerCase().includes(query) ||
                    endpoint.url.toLowerCase().includes(query) ||
                    endpoint.method.toLowerCase().includes(query),
            );

            if (collectionMatches) return collection;

            if (matchingEndpoints.length > 0) {
                return {
                    ...collection,
                    endpoints: matchingEndpoints,
                    isOpen: true,
                };
            }

            return null;
        })
        .filter((c): c is any => c !== null);
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
    <div class="flex flex-col h-full bg-muted/5 border-r select-none">
        <!-- Header / Search -->
        <div class="p-3 border-b flex items-center justify-between">
            <div class="relative flex-1">
                <Search
                    class="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground"
                />
                <input
                    ref="searchInput"
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search all... (Cmd+K)"
                    class="w-full bg-background border rounded-md py-1.5 pl-8 pr-8 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
                />
                <button
                    v-if="searchQuery"
                    @click="searchQuery = ''"
                    class="absolute right-2 top-2 p-0.5 hover:bg-muted rounded-sm transition-colors"
                >
                    <X class="h-3 w-3 text-muted-foreground" />
                </button>
            </div>
        </div>

        <!-- Scrollable content showing two collapsible sections -->
        <div class="flex-1 overflow-y-auto space-y-4 py-3">
            <!-- SERVICES SECTION -->
            <div class="flex flex-col">
                <div class="px-3 py-1.5 flex items-center justify-between hover:bg-muted/30 transition-colors group cursor-pointer" @click="isServicesExpanded = !isServicesExpanded">
                    <div class="flex items-center gap-2 text-muted-foreground hover:text-foreground">
                        <component :is="isServicesExpanded ? ChevronDown : ChevronRight" class="h-4 w-4" />
                        <Folder class="h-4 w-4 text-sky-500" />
                        <span class="text-xs font-semibold uppercase tracking-wider">Services</span>
                        <span class="text-[10px] bg-muted px-1.5 py-0.5 rounded-full font-medium text-muted-foreground">
                            {{ servicesStore.services.length }}
                        </span>
                    </div>

                    <!-- Services Actions (stop click propagation so we don't collapse) -->
                    <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity" @click.stop>
                        <Popover v-model:open="isImportPopoverOpen">
                            <PopoverTrigger as-child>
                                <button
                                    class="p-1 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
                                    title="Import options"
                                >
                                    <Download class="h-3.5 w-3.5" />
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="w-48 p-1" align="start">
                                <div class="flex flex-col">
                                    <button
                                        @click="
                                            isImportPopoverOpen = false;
                                            handleImportService();
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>From Directory</span>
                                    </button>
                                    <button
                                        @click="
                                            isImportPopoverOpen = false;
                                            isSwaggerDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>Swagger / OpenAPI</span>
                                    </button>
                                    <button
                                        @click="
                                            isImportPopoverOpen = false;
                                            isCurlDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>cURL Command</span>
                                    </button>
                                </div>
                            </PopoverContent>
                        </Popover>

                        <Popover v-model:open="isAddPopoverOpen">
                            <PopoverTrigger as-child>
                                <button
                                    class="p-1 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
                                    title="Add service/endpoint"
                                >
                                    <Plus class="h-3.5 w-3.5" />
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="w-48 p-1" align="end">
                                <div class="flex flex-col">
                                    <button
                                        @click="
                                            isAddPopoverOpen = false;
                                            isServiceDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>Add New Service</span>
                                    </button>
                                    <button
                                        @click="
                                            isAddPopoverOpen = false;
                                            isEndpointDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>Add New Endpoint</span>
                                    </button>
                                    <button
                                        @click="
                                            isAddPopoverOpen = false;
                                            handleImportService();
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>Import from Directory</span>
                                    </button>
                                    <button
                                        @click="
                                            isAddPopoverOpen = false;
                                            isSwaggerDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <span>Import from Swagger</span>
                                    </button>
                                </div>
                            </PopoverContent>
                        </Popover>
                    </div>
                </div>

                <!-- Services List -->
                <div v-show="isServicesExpanded" class="px-2 mt-1">
                    <div v-if="filteredServices.length === 0" class="text-xs text-muted-foreground p-3 text-center">
                        No services found
                    </div>
                    <ServiceTree
                        v-else
                        :services="filteredServices"
                        @select-endpoint="emit('select-endpoint', $event)"
                        @select-service-settings="emit('select-service-settings', $event)"
                        @env-change="(id, env) => emit('env-change', id, env)"
                        @import-curl="emit('import-curl', $event)"
                    />
                </div>
            </div>

            <!-- SCRATCHPAD SECTION -->
            <div class="flex flex-col">
                <div class="px-3 py-1.5 flex items-center justify-between hover:bg-muted/30 transition-colors group cursor-pointer" @click="isScratchpadExpanded = !isScratchpadExpanded">
                    <div class="flex items-center gap-2 text-muted-foreground hover:text-foreground">
                        <component :is="isScratchpadExpanded ? ChevronDown : ChevronRight" class="h-4 w-4" />
                        <Layers class="h-4 w-4 text-emerald-500" />
                        <span class="text-xs font-semibold uppercase tracking-wider">Scratchpad</span>
                        <span class="text-[10px] bg-muted px-1.5 py-0.5 rounded-full font-medium text-muted-foreground">
                            {{ collectionsStore.collections.length }}
                        </span>
                    </div>

                    <!-- Scratchpad Actions -->
                    <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity" @click.stop>
                        <Popover v-model:open="isCollectionPopoverOpen">
                            <PopoverTrigger as-child>
                                <button
                                    class="p-1 hover:bg-muted rounded-md transition-colors text-muted-foreground hover:text-foreground"
                                    title="Add collection/endpoint"
                                >
                                    <Plus class="h-3.5 w-3.5" />
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="w-48 p-1" align="end">
                                <div class="flex flex-col">
                                    <button
                                        @click="
                                            isCollectionPopoverOpen = false;
                                            isCollectionDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <Layers class="h-3.5 w-3.5 text-primary" />
                                        <span>New Collection</span>
                                    </button>
                                    <button
                                        @click="
                                            isCollectionPopoverOpen = false;
                                            isCollectionEndpointDialogOpen = true;
                                        "
                                        class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-xs text-left transition-colors"
                                    >
                                        <PlusCircle class="h-3.5 w-3.5 text-green-500" />
                                        <span>New Endpoint</span>
                                    </button>
                                </div>
                            </PopoverContent>
                        </Popover>
                    </div>
                </div>

                <!-- Scratchpad List -->
                <div v-show="isScratchpadExpanded" class="px-2 mt-1">
                    <div v-if="filteredCollections.length === 0" class="text-xs text-muted-foreground p-3 text-center">
                        No collections found
                    </div>
                    <ServiceTree
                        v-else
                        :services="filteredCollections"
                        @select-endpoint="emit('select-endpoint', $event)"
                        @select-service-settings="emit('select-service-settings', $event)"
                        @env-change="(id, env) => emit('env-change', id, env)"
                        @endpoint-context="(e, ep) => emit('endpoint-context', e, ep)"
                    />
                </div>
            </div>
        </div>
    </div>
</template>

