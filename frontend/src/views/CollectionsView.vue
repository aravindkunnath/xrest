<script setup lang="ts">
import CollectionExplorer from "@/components/collections/CollectionExplorer.vue";
import AddCollectionDialog from "@/components/dialogs/AddCollectionDialog.vue";
import AddCollectionEndpointDialog from "@/components/dialogs/AddCollectionEndpointDialog.vue";
import AddToServiceDialog from "@/components/dialogs/AddToServiceDialog.vue";
import ShareRequestDialog from "@/components/dialogs/ShareRequestDialog.vue";
import UnsafeEnvironmentDialog from "@/components/dialogs/UnsafeEnvironmentDialog.vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import RequestWorkspace from "@/components/workspace/RequestWorkspace.vue";
import { useDialogState } from "@/composables/useDialogState";
import { useEnvironmentVariables } from "@/composables/useEnvironmentVariables";
import { useRequestExecution } from "@/composables/useRequestExecution";
import { useTabManager } from "@/composables/useTabManager";
import { useCollectionsStore } from "@/stores/collections";
import { useServicesStore } from "@/stores/services";
import { onMounted, onUnmounted, ref } from "vue";
import { toast } from "vue-sonner";

const sharingTabData = ref<any>(null);

const collectionsStore = useCollectionsStore();
const servicesStore = useServicesStore();
const {
  isCollectionDialogOpen,
  isCollectionEndpointDialogOpen,
  openCollectionEndpointDialog,
  isShareDialogOpen,
  isUnsafeDialogOpen,
} = useDialogState();

const { getEnvName } = useEnvironmentVariables();

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

onMounted(async () => {
  await collectionsStore.loadCollections();
  await servicesStore.loadServices();
  await initializeTabsFromSavedState();
  window.addEventListener("click", closeContextMenu);
});

onUnmounted(() => {
  window.removeEventListener("click", closeContextMenu);
});

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
  // Find source collection
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
    // 1. Add endpoint to service
    const serviceIndex = servicesStore.services.findIndex(
      (s) => s.id === targetServiceId,
    );
    if (serviceIndex === -1) return;

    const service = servicesStore.services[serviceIndex];

    // Merge variables into ALL environments of the target service
    const updatedService = {
      ...service,
      environments: service.environments.map((env) => ({
        ...env,
        variables: [
          ...env.variables,
          ...newVariables.filter(
            (nv) => !env.variables.some((ev) => ev.name === nv.name),
          ),
        ],
      })),
      endpoints: [...service.endpoints, updatedEndpoint],
    };

    await servicesStore.updateService(serviceIndex, updatedService);

    // 2. Remove endpoint from collection
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

    // 3. Close tab if it was open
    closeTab(`endpoint-${updatedEndpoint.id}`);

    toast.success(
      `"${updatedEndpoint.name}" moved to service "${service.name}"`,
    );
  } catch (err) {
    console.error("Migration failed:", err);
    toast.error("Failed to migrate endpoint");
  }
};

const handleSelectEndpoint = (endpoint: any) => {
  const collection = collectionsStore.collections.find((c) =>
    c.endpoints.some((e) => e.id === endpoint.id),
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

    addTab({
      id: `endpoint-${endpoint.id}`,
      type: "request",
      endpointId: endpoint.id,
      serviceId: collection?.id,
      title: endpoint.name,
      method: endpoint.method,
      url: endpoint.url,
      params: initialParams,
      headers: initialHeaders,
      body: {
        type: "application/json",
        content: endpoint.body || "",
      },
      auth: {
        type: "none",
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

const handleSelectCollectionSettings = (collection: any) => {
  const tabId = `settings-${collection.id}`;
  const existingTab = tabs.value.find((t) => t.id === tabId);
  if (existingTab) {
    activeTab.value = tabId;
  } else {
    addTab({
      id: tabId,
      title: `${collection.name}`,
      type: "settings",
      serviceId: collection.id,
      serviceData: JSON.parse(JSON.stringify(collection)),
    });
  }
};

const handleCollectionCreated = (collection: any) => {
  collectionsStore.addCollection(collection);
};

const handleEndpointCreated = async (endpoint: any, collectionId: string) => {
  let finalCollectionId = collectionId;
  let collectionIndex = collectionsStore.collections.findIndex(
    (c) => c.id === collectionId,
  );

  if (collectionIndex === -1) {
    // Create a default collection if it doesn't exist or none was selected
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
          variables: [{ name: "BASE_URL", value: "https://api.example.com" }],
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

const handleShareRequest = (tab: any) => {
  sharingTabData.value = tab;
  isShareDialogOpen.value = true;
};

// Workspace Event Handlers
const handleSaveRequest = async (payload: {
  serviceIndex: number;
  updatedItem: any;
  tab: any;
}) => {
  try {
    await collectionsStore.updateCollection(
      payload.serviceIndex,
      payload.updatedItem,
    );
    toast.success("Endpoint saved", {
      description: `Changes to "${payload.tab.title}" have been persisted.`,
    });
  } catch (error) {
    toast.error("Failed to save endpoint");
  }
};


</script>

<template>
  <div class="flex-1 overflow-hidden h-full">
    <ResizablePanelGroup direction="horizontal">
      <!-- Left Resizable Sidebar -->
      <ResizablePanel :default-size="20" :min-size="15" class="bg-muted/5 flex flex-col h-full border-r">
        <CollectionExplorer @select-endpoint="handleSelectEndpoint"
          @select-service-settings="handleSelectCollectionSettings"
          @env-change="collectionsStore.setSelectedEnvironment" @endpoint-context="handleEndpointContext" />
      </ResizablePanel>

      <ResizableHandle with-handle />

      <ResizablePanel :default-size="80" class="flex flex-col h-full">
        <RequestWorkspace :items="collectionsStore.collections" label="Collection"
          :on-new-request="openCollectionEndpointDialog"
          @share-request="handleShareRequest"
          @save-request="handleSaveRequest" />
      </ResizablePanel>
    </ResizablePanelGroup>

    <!-- Dialogs -->
    <AddCollectionDialog :open="isCollectionDialogOpen" @update:open="isCollectionDialogOpen = $event"
      @collection-created="handleCollectionCreated" />

    <AddCollectionEndpointDialog :open="isCollectionEndpointDialogOpen" :collections="collectionsStore.collections"
      @update:open="isCollectionEndpointDialogOpen = $event" @endpoint-created="handleEndpointCreated" />

    <ShareRequestDialog :open="isShareDialogOpen" :tab="sharingTabData" @update:open="isShareDialogOpen = $event" />

    <UnsafeEnvironmentDialog :open="isUnsafeDialogOpen" :environment-name="unsafeTabToProceed ? getEnvName(unsafeTabToProceed) : ''
      " :countdown="unsafeCountdown" @update:open="isUnsafeDialogOpen = $event" @proceed="proceedWithUnsafeRequest"
      @cancel="cancelUnsafeRequest" />

    <AddToServiceDialog :open="isAddToServiceDialogOpen" @update:open="isAddToServiceDialogOpen = $event"
      :endpoint="endpointToMigrate" :source-collection="sourceCollectionMigrate" @added="handleEndpointMigrated" />

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.show" :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
        class="fixed z-[100] bg-popover text-popover-foreground border shadow-md rounded-md p-1 min-w-[150px] animate-in fade-in zoom-in-95 duration-100">
        <button @click="openMigrateDialog"
          class="flex w-full items-center gap-2 px-2.5 py-1.5 hover:bg-accent hover:text-accent-foreground rounded-sm transition-colors">
          <ArrowRight class="h-3.5 w-3.5 text-primary" />
          Add to Service
        </button>
      </div>
    </Teleport>
  </div>
</template>
