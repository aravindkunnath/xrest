<script setup lang="ts">
import { onMounted } from "vue";
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

// Dialogs
import AddServiceDialog from "@/components/dialogs/AddServiceDialog.vue";
import AddEndpointDialog from "@/components/dialogs/AddEndpointDialog.vue";
import SwaggerImportDialog from "@/components/dialogs/SwaggerImportDialog.vue";
import ShareRequestDialog from "@/components/dialogs/ShareRequestDialog.vue";
import UnsafeEnvironmentDialog from "@/components/dialogs/UnsafeEnvironmentDialog.vue";
import ImportCurlDialog from "@/components/dialogs/ImportCurlDialog.vue";
import { ref } from "vue";

const servicesStore = useServicesStore();
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
} = useDialogState();

const { allActiveVariables, activeEnvironments, getEnvName } =
  useEnvironmentVariables();

const { gitStatuses, handleSyncGit, handleInitGit } = useGitIntegration();

const { tabs, activeTab, addTab, initializeTabsFromSavedState } =
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
  await servicesStore.loadServices();
  // Also load collections so variable interpolation works for collection-based tabs
  await useCollectionsStore().loadCollections();
  await initializeTabsFromSavedState();
});

// Event Handlers for child components
const handleServiceCreated = (service: any) => {
  servicesStore.addService(service);
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
    // Pass serviceId so the new tab gets variables even on first render
    handleSelectEndpoint(endpoint, serviceId);
    isEndpointDialogOpen.value = false;
  }
};

const handleSwaggerImportComplete = async () => {
  await servicesStore.loadServices();
};

const handleSelectEndpoint = (endpoint: any, knownServiceId?: string) => {
  // Find service that owns this endpoint (or use knownServiceId when opening a newly created endpoint)
  const service = knownServiceId
    ? servicesStore.services.find((s) => s.id === knownServiceId)
    : servicesStore.services.find((s) =>
        s.endpoints.some((e) => e.id === endpoint.id),
      );
  // Determine if service has BASE_URL defined to use placeholders
  const hasBaseUrl = service?.environments?.some((e) =>
    e.variables.some((v) => v.name === "BASE_URL"),
  );

  const existingTab = tabs.value.find(
    (t) => t.id === `endpoint-${endpoint.id}`,
  );
  if (existingTab) {
    activeTab.value = existingTab.id;
  } else {
    // Prepare initial state from endpoint and service
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
    // Use placeholder instead of literal value for better experience
    const fullUrl =
      hasBaseUrl && endpoint.url.startsWith("/")
        ? `{{BASE_URL}}${path}`
        : endpoint.url;

    // Initialize preflight from endpoint.preflight (for UI editing)
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
        type: service?.isAuthenticated
          ? service.authType?.toLowerCase() || "none"
          : "none",
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
      serviceData: JSON.parse(JSON.stringify(service)), // Deep copy for editing
    });
    // Git status is fetched inside Workspace/ServiceSettingsView when active
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
    await servicesStore.updateService(
      payload.serviceIndex,
      payload.updatedItem,
      `Update endpoint: ${payload.tab.title}`
    );

    // Sync versions for tab
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
        <ServiceExplorer @select-endpoint="handleSelectEndpoint" @select-service-settings="handleSelectServiceSettings"
          @env-change="servicesStore.setSelectedEnvironment" @import-curl="handleImportCurl" />
      </ResizablePanel>

      <ResizableHandle with-handle />

      <!-- Workspace Component -->
      <ResizablePanel :default-size="80">
        <RequestWorkspace :items="servicesStore.services" :git-statuses="gitStatuses" label="Service"
          :on-new-request="openEndpointDialog"
          @sync-git="handleSyncGit" @init-git="handleInitGit" @share-request="handleShareRequest"
          @save-request="handleSaveRequest" />
      </ResizablePanel>
    </ResizablePanelGroup>

    <!-- Dialogs -->
    <AddServiceDialog :open="isServiceDialogOpen" @update:open="isServiceDialogOpen = $event"
      @service-created="handleServiceCreated" />

    <AddEndpointDialog :open="isEndpointDialogOpen" :services="servicesStore.services"
      :all-active-variables="allActiveVariables" :active-environments="activeEnvironments"
      @update:open="isEndpointDialogOpen = $event" @endpoint-created="handleEndpointCreated" />

    <SwaggerImportDialog :open="isSwaggerDialogOpen" @update:open="isSwaggerDialogOpen = $event"
      @import-complete="handleSwaggerImportComplete" />

    <ShareRequestDialog :open="isShareDialogOpen" :tab="sharingTabData" @update:open="isShareDialogOpen = $event" />

    <UnsafeEnvironmentDialog :open="isUnsafeDialogOpen" :environment-name="unsafeTabToProceed ? getEnvName(unsafeTabToProceed) : ''
      " :countdown="unsafeCountdown" @update:open="isUnsafeDialogOpen = $event" @proceed="proceedWithUnsafeRequest"
      @cancel="cancelUnsafeRequest" />
    <ImportCurlDialog :open="isCurlDialogOpen" :service-id="selectedImportServiceId"
      @update:open="isCurlDialogOpen = $event" @import-complete="handleSwaggerImportComplete" />
  </div>
</template>
