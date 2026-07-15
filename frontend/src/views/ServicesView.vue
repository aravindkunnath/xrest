<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
    ResizableHandle,
    ResizablePanel,
    ResizablePanelGroup,
} from "@/components/ui/resizable";

// Stores & Composables
import { useServicesStore } from "@/stores/services";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogState } from "@/composables/useDialogState";
import { useEnvironmentVariables } from "@/composables/useEnvironmentVariables";
import { useGitIntegration } from "@/composables/useGitIntegration";
import { useTabManager } from "@/composables/useTabManager";
import { useRequestExecution } from "@/composables/useRequestExecution";

// New Components
import ServiceExplorer from "@/components/services/ServiceExplorer.vue";
import RequestWorkspace from "@/components/workspace/RequestWorkspace.vue";
import { toast } from "vue-sonner";
import { ArrowRight } from "@lucide/vue";

// Dialogs
import AddServiceDialog from "@/components/dialogs/AddServiceDialog.vue";
import AddEndpointDialog from "@/components/dialogs/AddEndpointDialog.vue";
import SwaggerImportDialog from "@/components/dialogs/SwaggerImportDialog.vue";
import ShareRequestDialog from "@/components/dialogs/ShareRequestDialog.vue";
import UnsafeEnvironmentDialog from "@/components/dialogs/UnsafeEnvironmentDialog.vue";
import ImportCurlDialog from "@/components/dialogs/ImportCurlDialog.vue";
import AddCollectionDialog from "@/components/dialogs/AddCollectionDialog.vue";
import AddCollectionEndpointDialog from "@/components/dialogs/AddCollectionEndpointDialog.vue";
import AddToServiceDialog from "@/components/dialogs/AddToServiceDialog.vue";

const servicesStore = useServicesStore();
const collectionsStore = useCollectionsStore();
const sharingTabData = ref<any>(null);
const selectedImportServiceId = ref<string>("");

const {
    isServiceDialogOpen,
    isEndpointDialogOpen,
    openEndpointDialog,
    isSwaggerDialogOpen,
    isShareDialogOpen,
    isUnsafeDialogOpen,
    isCurlDialogOpen,
    isCollectionDialogOpen,
    isCollectionEndpointDialogOpen,
    // openCollectionEndpointDialog,
} = useDialogState();

const { allActiveVariables, activeEnvironments, getEnvName } =
    useEnvironmentVariables();

const { gitStatuses, handleSyncGit, handleInitGit } = useGitIntegration();

const { tabs, activeTab, addTab, closeTab, initializeTabsFromSavedState } =
    useTabManager();

const {
    unsafeTabToProceed,
    unsafeCountdown,
    handleSendRequest,
    proceedWithUnsafeRequest: proceedUnsafe,
    cancelUnsafeRequest,
} = useRequestExecution(isUnsafeDialogOpen);

const proceedWithUnsafeRequest = () => {
    proceedUnsafe(handleSendRequest);
};

// Context Menu / Migration State
const isAddToServiceDialogOpen = ref(false);
const endpointToMigrate = ref<any>(null);
const sourceCollectionMigrate = ref<any>(null);

const contextMenu = ref({
    show: false,
    x: 0,
    y: 0,
    endpoint: null as any,
});

const handleEndpointContext = (e: MouseEvent, endpoint: any) => {
    contextMenu.value = {
        show: true,
        x: e.clientX,
        y: e.clientY,
        endpoint: endpoint,
    };
};

const closeContextMenu = () => {
    contextMenu.value.show = false;
};

const openMigrateDialog = () => {
    if (!contextMenu.value.endpoint) return;

    endpointToMigrate.value = contextMenu.value.endpoint;
    sourceCollectionMigrate.value = collectionsStore.collections.find((c) =>
        c.endpoints.some((e) => e.id === contextMenu.value.endpoint.id),
    );

    isAddToServiceDialogOpen.value = true;
    closeContextMenu();
};

const handleEndpointMigrated = async (
    targetServiceId: string,
    updatedEndpoint: any,
    newVariables: any[],
) => {
    try {
        const serviceIndex = servicesStore.services.findIndex(
            (s) => s.id === targetServiceId,
        );
        if (serviceIndex === -1) return;

        const service = servicesStore.services[serviceIndex];

        const updatedService = {
            ...service,
            environments: service.environments.map((env) => ({
                ...env,
                variables: [
                    ...env.variables,
                    ...newVariables.filter(
                        (nv) =>
                            !env.variables.some((ev) => ev.name === nv.name),
                    ),
                ],
            })),
            endpoints: [...service.endpoints, updatedEndpoint],
        };

        await servicesStore.updateService(serviceIndex, updatedService);

        const collectionIndex = collectionsStore.collections.findIndex((c) =>
            c.endpoints.some((e) => e.id === updatedEndpoint.id),
        );
        if (collectionIndex !== -1) {
            const collection = collectionsStore.collections[collectionIndex];
            const updatedCollection = {
                ...collection,
                endpoints: collection.endpoints.filter(
                    (e) => e.id !== updatedEndpoint.id,
                ),
            };
            await collectionsStore.updateCollection(
                collectionIndex,
                updatedCollection,
            );
        }

        closeTab(`endpoint-${updatedEndpoint.id}`);
        toast.success(
            `"${updatedEndpoint.name}" moved to service "${service.name}"`,
        );
    } catch (err) {
        console.error("Migration failed:", err);
        toast.error("Failed to migrate endpoint");
    }
};

onMounted(async () => {
    await servicesStore.loadServices();
    await collectionsStore.loadCollections();
    await initializeTabsFromSavedState();
    window.addEventListener("click", closeContextMenu);
});

onUnmounted(() => {
    window.removeEventListener("click", closeContextMenu);
});

// Event Handlers for child components
const handleServiceCreated = (service: any) => {
    servicesStore.addService(service);
};

const handleCollectionCreated = (collection: any) => {
    collectionsStore.addCollection(collection);
};

const handleCollectionEndpointCreated = async (
    endpoint: any,
    collectionId: string,
) => {
    let finalCollectionId = collectionId;
    let collectionIndex = collectionsStore.collections.findIndex(
        (c) => c.id === collectionId,
    );

    if (collectionIndex === -1) {
        const newCollection: any = {
            id: `c-${Date.now()}`,
            name: "My Collection",
            directory: "",
            isAuthenticated: false,
            authType: "none",
            endpoints: [],
            environments: [
                {
                    name: "GLOBAL",
                    isUnsafe: false,
                    variables: [
                        { name: "BASE_URL", value: "https://api.example.com" },
                    ],
                },
            ],
            selectedEnvironment: "GLOBAL",
        };
        await collectionsStore.addCollection(newCollection);
        finalCollectionId = newCollection.id;
        collectionIndex = collectionsStore.collections.length - 1;
    }

    const collection = collectionsStore.collections[collectionIndex];
    const updatedCollection = {
        ...collection,
        endpoints: [
            ...collection.endpoints,
            { ...endpoint, serviceId: finalCollectionId },
        ],
    };
    await collectionsStore.updateCollection(collectionIndex, updatedCollection);
    handleSelectEndpoint(
        updatedCollection.endpoints[updatedCollection.endpoints.length - 1],
    );
};

const handleEndpointCreated = async (endpoint: any, serviceId: string) => {
    const serviceIndex = servicesStore.services.findIndex(
        (s) => s.id === serviceId,
    );
    if (serviceIndex !== -1) {
        const service = servicesStore.services[serviceIndex];
        const updatedService = {
            ...service,
            endpoints: [...service.endpoints, endpoint],
        };
        await servicesStore.updateService(
            serviceIndex,
            updatedService,
            `Create endpoint: ${endpoint.name}`,
        );
        handleSelectEndpoint(endpoint, serviceId);
        isEndpointDialogOpen.value = false;
    }
};

const handleSwaggerImportComplete = async () => {
    await servicesStore.loadServices();
};

const handleSelectEndpoint = (endpoint: any, knownServiceId?: string) => {
    const service = knownServiceId
        ? servicesStore.services.find((s) => s.id === knownServiceId) ||
          collectionsStore.collections.find((c) => c.id === knownServiceId)
        : servicesStore.services.find((s) =>
              s.endpoints.some((e) => e.id === endpoint.id),
          ) ||
          collectionsStore.collections.find((c) =>
              c.endpoints.some((e) => e.id === endpoint.id),
          );

    const hasBaseUrl = service?.environments?.some((e) =>
        e.variables?.some((v) => v.name === "BASE_URL"),
    );

    const existingTab = tabs.value.find(
        (t) => t.id === `endpoint-${endpoint.id}`,
    );
    if (existingTab) {
        activeTab.value = existingTab.id;
    } else {
        const initialParams =
            endpoint.params?.length > 0
                ? endpoint.params.map((p: any) => ({ ...p, enabled: true }))
                : [{ enabled: true, name: "", value: "" }];

        const initialHeaders =
            endpoint.headers?.length > 0
                ? endpoint.headers.map((h: any) => ({ ...h, enabled: true }))
                : [{ enabled: true, name: "", value: "" }];

        const path = endpoint.url.startsWith("/")
            ? endpoint.url
            : `/${endpoint.url}`;

        const fullUrl =
            hasBaseUrl && endpoint.url.startsWith("/")
                ? `{{BASE_URL}}${path}`
                : endpoint.url;

        const preflightConfig = endpoint.preflight || {
            enabled: false,
            method: "POST",
            url: "",
            body: "",
            headers: [],
            cacheToken: true,
            cacheDuration: "derived",
            cacheDurationKey: "expires_in",
            cacheDurationUnit: "seconds",
            tokenKey: "access_token",
            tokenHeader: "Authorization",
        };

        const authType =
            service && "isAuthenticated" in service && service.isAuthenticated
                ? service.authType?.toLowerCase() || "none"
                : "none";

        addTab({
            id: `endpoint-${endpoint.id}`,
            type: "request",
            endpointId: endpoint.id,
            serviceId: service?.id,
            title: endpoint.name,
            method: endpoint.method,
            url: fullUrl,
            params: initialParams,
            headers: initialHeaders,
            body: {
                type: "application/json",
                content: endpoint.body || "",
            },
            auth: {
                type: authType,
                active: true,
                bearerToken: "",
                basicUser: "",
                basicPass: "",
                apiKeyName: "",
                apiKeyValue: "",
                apiKeyLocation: "header",
            },
            preflight: preflightConfig,
            versions: endpoint.versions || [],
        });
    }
};

const handleSelectServiceSettings = (service: any) => {
    const tabId = `settings-${service.id}`;
    const existingTab = tabs.value.find((t) => t.id === tabId);
    if (existingTab) {
        activeTab.value = tabId;
    } else {
        addTab({
            id: tabId,
            title: `${service.name}`,
            type: "settings",
            serviceId: service.id,
            serviceData: JSON.parse(JSON.stringify(service)),
        });
    }
};

const handleShareRequest = (tab: any) => {
    sharingTabData.value = tab;
    isShareDialogOpen.value = true;
};

const handleImportCurl = (service: any) => {
    selectedImportServiceId.value = service?.id || "";
    isCurlDialogOpen.value = true;
};

const handleSaveRequest = async (payload: {
    serviceIndex: number;
    updatedItem: any;
    tab: any;
}) => {
    try {
        if (
            payload.tab.serviceId?.startsWith("c-") ||
            !servicesStore.services[payload.serviceIndex]
        ) {
            const colIdx = collectionsStore.collections.findIndex(
                (c) => c.id === payload.tab.serviceId,
            );
            if (colIdx !== -1) {
                await collectionsStore.updateCollection(
                    colIdx,
                    payload.updatedItem,
                );
                toast.success("Endpoint saved", {
                    description: `Changes to "${payload.tab.title}" have been persisted.`,
                });
                return;
            }
        }

        await servicesStore.updateService(
            payload.serviceIndex,
            payload.updatedItem,
            `Update endpoint: ${payload.tab.title}`,
        );

        const finalService = servicesStore.services[payload.serviceIndex];
        const finalEndpoint = finalService?.endpoints.find(
            (e) => e.id === payload.tab.endpointId,
        );
        if (finalEndpoint) {
            payload.tab.versions = finalEndpoint.versions || [];
        }

        toast.success("Endpoint saved", {
            description: `Changes to "${payload.tab.title}" have been persisted.`,
        });
    } catch (error) {
        toast.error("Failed to save endpoint");
        console.error(error);
    }
};
</script>

<template>
    <div class="flex-1 overflow-hidden h-full">
        <ResizablePanelGroup direction="horizontal">
            <!-- Sidebar Component -->
            <ResizablePanel :default-size="20" :min-size="15">
                <ServiceExplorer
                    @select-endpoint="handleSelectEndpoint"
                    @select-service-settings="handleSelectServiceSettings"
                    @env-change="servicesStore.setSelectedEnvironment"
                    @import-curl="handleImportCurl"
                    @endpoint-context="handleEndpointContext"
                />
            </ResizablePanel>

            <ResizableHandle with-handle />

            <!-- Workspace Component -->
            <ResizablePanel :default-size="80">
                <RequestWorkspace
                    :items="servicesStore.services"
                    :git-statuses="gitStatuses"
                    label="Service"
                    :on-new-request="
                        servicesStore.services.length > 0
                            ? openEndpointDialog
                            : undefined
                    "
                    @sync-git="handleSyncGit"
                    @init-git="handleInitGit"
                    @share-request="handleShareRequest"
                    @save-request="handleSaveRequest"
                />
            </ResizablePanel>
        </ResizablePanelGroup>

        <!-- Dialogs -->
        <AddServiceDialog
            :open="isServiceDialogOpen"
            @update:open="isServiceDialogOpen = $event"
            @service-created="handleServiceCreated"
        />

        <AddEndpointDialog
            :open="isEndpointDialogOpen"
            :services="servicesStore.services"
            :all-active-variables="allActiveVariables"
            :active-environments="activeEnvironments"
            @update:open="isEndpointDialogOpen = $event"
            @endpoint-created="handleEndpointCreated"
        />

        <SwaggerImportDialog
            :open="isSwaggerDialogOpen"
            @update:open="isSwaggerDialogOpen = $event"
            @import-complete="handleSwaggerImportComplete"
        />

        <ShareRequestDialog
            :open="isShareDialogOpen"
            :tab="sharingTabData"
            @update:open="isShareDialogOpen = $event"
        />

        <UnsafeEnvironmentDialog
            :open="isUnsafeDialogOpen"
            :environment-name="
                unsafeTabToProceed ? getEnvName(unsafeTabToProceed) : ''
            "
            :countdown="unsafeCountdown"
            @update:open="isUnsafeDialogOpen = $event"
            @proceed="proceedWithUnsafeRequest"
            @cancel="cancelUnsafeRequest"
        />
        <ImportCurlDialog
            :open="isCurlDialogOpen"
            :service-id="selectedImportServiceId"
            @update:open="isCurlDialogOpen = $event"
            @import-complete="handleSwaggerImportComplete"
        />

        <AddCollectionDialog
            :open="isCollectionDialogOpen"
            @update:open="isCollectionDialogOpen = $event"
            @collection-created="handleCollectionCreated"
        />

        <AddCollectionEndpointDialog
            :open="isCollectionEndpointDialogOpen"
            :collections="collectionsStore.collections"
            @update:open="isCollectionEndpointDialogOpen = $event"
            @endpoint-created="handleCollectionEndpointCreated"
        />

        <AddToServiceDialog
            :open="isAddToServiceDialogOpen"
            @update:open="isAddToServiceDialogOpen = $event"
            :endpoint="endpointToMigrate"
            :source-collection="sourceCollectionMigrate"
            @added="handleEndpointMigrated"
        />

        <!-- Context Menu -->
        <Teleport to="body">
            <div
                v-if="contextMenu.show"
                :style="{
                    top: contextMenu.y + 'px',
                    left: contextMenu.x + 'px',
                }"
                class="fixed z-[100] bg-popover text-popover-foreground border shadow-md rounded-md p-1 min-w-[150px] animate-in fade-in zoom-in-95 duration-100"
            >
                <button
                    @click="openMigrateDialog"
                    class="flex w-full items-center gap-2 px-2.5 py-1.5 hover:bg-accent hover:text-accent-foreground rounded-sm transition-colors text-xs"
                >
                    <ArrowRight class="h-3.5 w-3.5 text-primary" />
                    Add to Service
                </button>
            </div>
        </Teleport>
    </div>
</template>
