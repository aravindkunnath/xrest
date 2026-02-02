<script setup lang="ts">
import { Plus, X, Settings2, Play } from "lucide-vue-next";
import { computed, onMounted, onUnmounted } from "vue";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// Components
import RequestUrlBar from "@/components/RequestUrlBar.vue";
import RequestParameters from "@/components/RequestParameters.vue";
import RequestBody from "@/components/RequestBody.vue";
import RequestHistory from "@/components/RequestHistory.vue";
import ResponseViewer from "@/components/ResponseViewer.vue";
import ServiceSettingsView from "@/components/ServiceSettingsView.vue";

// Composables & Utils
import { useTabManager } from "@/composables/useTabManager";
import { useEnvironmentVariables } from "@/composables/useEnvironmentVariables";
import { useRequestExecution } from "@/composables/useRequestExecution";
import { useDialogState } from "@/composables/useDialogState";
import { useServiceSettings } from "@/composables/useServiceSettings";
import { toast } from "vue-sonner";

const props = defineProps<{
  items: any[]; // Services or Collections
  gitStatuses?: Record<string, any>;
  label?: string; // 'Service' or 'Collection'
  /** When set, "New Request" / plus opens this (e.g. Add Endpoint dialog) instead of a blank tab */
  onNewRequest?: () => void;
}>();

const emit = defineEmits<{
  (e: "sync-git", serviceId: string, directory: string): void;
  (e: "init-git", serviceId: string, directory: string, url?: string): void;
  (e: "share-request", tab: any): void;
  (
    e: "save-request",
    payload: { serviceIndex: number; updatedItem: any; tab: any },
  ): void;
}>();

// Use tab manager
const {
  tabs,
  activeTab,
  addTab,
  closeTab,
  updateTabSnapshot,
} = useTabManager();

const { getTabVariables, isUnsafeEnv, getEnvName } =
  useEnvironmentVariables();

// Explicit computed so RequestUrlBar/InterpolatedInput get updated variables
// when store loads (first paint can happen before loadServices() completes).
const tabVariablesMap = computed(() => {
  const map: Record<string, Record<string, string>> = {};
  for (const tab of tabs.value) {
    map[tab.id] = getTabVariables(tab);
  }
  return map;
});

const { isUnsafeDialogOpen, openServiceDialog } = useDialogState();

const { isSending, handleSendRequest } =
  useRequestExecution(isUnsafeDialogOpen);

const { saveSettings } = useServiceSettings();

const handleNewRequest = () => {
  if (props.onNewRequest) {
    props.onNewRequest();
  } else {
    addTab();
  }
};

const handleGlobalKeyDown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    const tab = tabs.value.find((t: any) => t.id === activeTab.value);
    if (tab) {
      if (tab.type === "settings") {
        saveSettings(tab);
      } else {
        handleSaveRequest(tab);
      }
    }
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

const handleSaveRequest = async (tab: any) => {
  if (!tab.endpointId) {
    toast.error("Cannot save: This tab is not linked to an endpoint");
    return;
  }

  // Find the service/collection and endpoint
  let itemIndex = -1;
  let endpointIndex = -1;

  for (let i = 0; i < props.items.length; i++) {
    const idx = props.items[i].endpoints.findIndex(
      (e: any) => e.id === tab.endpointId,
    );
    if (idx !== -1) {
      itemIndex = i;
      endpointIndex = idx;
      break;
    }
  }

  if (itemIndex === -1 || endpointIndex === -1) {
    toast.error("Endpoint not found");
    return;
  }

  const item = props.items[itemIndex];
  const endpoint = item.endpoints[endpointIndex];

  // Update endpoint details
  let newPath = tab.url;
  if (item.environments.length > 0) {
    for (const envGroup of item.environments) {
      for (const variable of envGroup.variables) {
        if (
          variable.name === "BASE_URL" &&
          variable.value &&
          newPath.startsWith(variable.value)
        ) {
          newPath = newPath.replace(variable.value, "");
          break;
        }
      }
    }
  }

  const updatedEndpoint = {
    ...endpoint,
    method: tab.method,
    url: newPath,
    params: tab.params.map(({ enabled, name, value }: any) => ({
      enabled,
      name,
      value,
    })),
    headers: tab.headers.map(({ enabled, name, value }: any) => ({
      enabled,
      name,
      value,
    })),
    body: tab.body.content,
    preflight: tab.preflight,
  };

  const updatedItem = {
    ...item,
    endpoints: item.endpoints.map((e: any, idx: number) =>
      idx === endpointIndex ? updatedEndpoint : e,
    ),
  };

  emit("save-request", {
    serviceIndex: itemIndex,
    updatedItem,
    tab,
  });

  updateTabSnapshot(tab);
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
  }
};

const handleUpdateBody = (content: string, tab: any) => {
  tab.body.content = content;
};
</script>

<template>
  <div class="flex-1 flex flex-col h-full bg-background overflow-hidden">
    <!-- Tabs Header -->
    <Tabs v-model="activeTab" class="flex-1 flex flex-col overflow-hidden">
      <div class="flex items-center border-b bg-muted/20 px-4 shrink-0">
        <TabsList class="h-12 bg-transparent p-0 gap-0">
          <div v-for="tab in tabs" :key="tab.id" class="group relative flex items-center">
            <TabsTrigger :value="tab.id" @mousedown.middle.prevent.stop="closeTab(tab.id)"
              @auxclick.middle.prevent.stop="closeTab(tab.id)" :class="[
                'h-12 px-4 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-background transition-none relative min-w-[120px] max-w-[200px] justify-start',
                tab.isEdited ? 'italic text-muted-foreground' : ''
              ]">
              <div class="flex items-center gap-2 overflow-hidden w-full">
                <span v-if="tab.type === 'request'" :class="[
                  'text-[10px] font-bold px-1 rounded flex-shrink-0',
                  tab.method === 'GET' ? 'text-green-500 bg-green-500/10' :
                    tab.method === 'POST' ? 'text-orange-500 bg-orange-500/10' :
                      tab.method === 'PUT' ? 'text-blue-500 bg-blue-500/10' :
                        tab.method === 'DELETE' ? 'text-red-500 bg-red-500/10' :
                          'text-purple-500 bg-purple-500/10'
                ]">
                  {{ tab.method }}
                </span>
                <span v-else class="flex-shrink-0">
                  <Settings2 class="h-3 w-3 text-muted-foreground" />
                </span>
                <span class="truncate text-sm">{{ tab.title }}</span>
                <span v-if="tab.isEdited" class="w-1.5 h-1.5 rounded-full bg-primary flex-shrink-0 ml-1"></span>
              </div>

              <!-- Close Button -->
              <button @click.stop="closeTab(tab.id)"
                class="ml-1 p-0.5 rounded-sm hover:bg-muted opacity-0 group-hover:opacity-100 transition-opacity">
                <X class="h-3 w-3" />
              </button>
            </TabsTrigger>
          </div>
        </TabsList>

        <button @click="handleNewRequest" class="ml-2 p-1.5 hover:bg-muted rounded-md text-muted-foreground transition-colors"
          title="New Request (Ctrl+N)">
          <Plus class="h-4 w-4" />
        </button>
      </div>

      <!-- Tab Contents -->
      <div class="flex-1 overflow-hidden">
        <TabsContent v-for="tab in tabs as any" :key="tab.id" :value="tab.id" class="h-full mt-0 focus-visible:ring-0">
          <!-- Request View -->
          <div v-if="tab.type === 'request'" class="h-full flex flex-col">
            <ResizablePanelGroup direction="vertical">
              <ResizablePanel :default-size="50" :min-size="30" class="flex flex-col overflow-hidden">
                <!-- URL Bar -->
                <div class="p-4 border-b shrink-0">
                  <RequestUrlBar v-model:method="tab.method" v-model:url="tab.url" :is-sending="isSending"
                    :is-unsafe="isUnsafeEnv(tab)" :variables="tabVariablesMap[tab.id] ?? {}" :environment-name="getEnvName(tab)"
                    @send="handleSendRequest(tab)" @save="handleSaveRequest(tab)" @share="emit('share-request', tab)" />
                </div>

                <!-- Request Options (Tabs) -->
                <div class="flex-1 overflow-hidden flex flex-col">
                  <Tabs v-model="tab.activeSubTab" class="flex-1 flex flex-col overflow-hidden">
                    <TabsList class="justify-start h-9 bg-transparent border-b px-4 rounded-none shrink-0">
                      <TabsTrigger value="params"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        Params
                      </TabsTrigger>
                      <TabsTrigger value="auth"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        Auth
                      </TabsTrigger>
                      <TabsTrigger value="headers"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        Headers
                      </TabsTrigger>
                      <TabsTrigger value="body"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        Body
                      </TabsTrigger>
                      <TabsTrigger value="history"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        History
                      </TabsTrigger>
                      <TabsTrigger value="preflight"
                        class="h-9 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent">
                        Preflight
                      </TabsTrigger>
                    </TabsList>

                    <div class="flex-1 overflow-auto">
                      <TabsContent value="params" class="p-0 m-0">
                        <RequestParameters :items="tab.params" :variables="tabVariablesMap[tab.id] ?? {}"
                          :environment-name="getEnvName(tab)" />
                      </TabsContent>
                      <TabsContent value="auth" class="p-4 m-0">
                        <div
                          class="text-sm text-muted-foreground bg-muted/30 p-4 rounded-lg border border-dashed text-center">
                          Authentication is managed at the {{ label?.toLowerCase() }} level.
                          <button
                            @click="handleSelectServiceSettings({ id: tab.serviceId, name: props.items.find(i => i.id === tab.serviceId)?.name })"
                            class="text-primary hover:underline ml-1">
                            Open Settings
                          </button>
                        </div>
                      </TabsContent>
                      <TabsContent value="headers" class="p-0 m-0">
                        <RequestParameters :items="tab.headers" :variables="tabVariablesMap[tab.id] ?? {}"
                          :environment-name="getEnvName(tab)" />
                      </TabsContent>
                      <TabsContent value="body" class="p-0 m-0 h-full">
                        <RequestBody :body="{ content: tab.body.content, type: tab.body.type }"
                          :variables="tabVariablesMap[tab.id] ?? {}" :environment-name="getEnvName(tab)"
                          @update:content="handleUpdateBody($event, tab)" />
                      </TabsContent>
                      <TabsContent value="history" class="p-0 m-0">
                        <RequestHistory
                          :tab-id="tab.id"
                          :service-id="tab.serviceId"
                          :endpoint-id="tab.endpointId"
                        />
                      </TabsContent>
                      <TabsContent value="preflight" class="p-0 m-0 overflow-visible">
                        <div class="p-4">
                          <div class="flex items-center justify-between mb-4">
                            <div>
                              <h3 class="text-sm font-medium">Pre-request Script (UI Config)</h3>
                              <p class="text-xs text-muted-foreground">Configure a request to run before this one (e.g.,
                                to fetch a token).</p>
                            </div>
                            <div class="flex items-center gap-2">
                              <span class="text-xs font-medium">{{ tab.preflight?.enabled ? 'Enabled' : 'Disabled'
                                }}</span>
                              <button @click="tab.preflight.enabled = !tab.preflight.enabled"
                                :class="['w-8 h-4 rounded-full transition-colors relative', tab.preflight?.enabled ? 'bg-primary' : 'bg-muted border']">
                                <div
                                  :class="['absolute top-0.5 w-3 h-3 rounded-full bg-white transition-all', tab.preflight?.enabled ? 'left-4.5' : 'left-0.5']">
                                </div>
                              </button>
                            </div>
                          </div>

                          <div v-if="tab.preflight?.enabled" class="space-y-4 animate-in fade-in slide-in-from-top-2">
                            <div class="grid grid-cols-4 gap-4">
                              <div class="col-span-1">
                                <label
                                  class="text-[10px] uppercase font-bold text-muted-foreground mb-1 block">Method</label>
                                <select v-model="tab.preflight.method"
                                  class="w-full bg-background border rounded px-2 py-1 text-sm h-8">
                                  <option>GET</option>
                                  <option>POST</option>
                                  <option>PUT</option>
                                </select>
                              </div>
                              <div class="col-span-3">
                                <label
                                  class="text-[10px] uppercase font-bold text-muted-foreground mb-1 block">URL</label>
                                <input v-model="tab.preflight.url" placeholder="https://auth.example.com/token"
                                  class="w-full bg-background border rounded px-2 py-1 text-sm h-8" />
                              </div>
                            </div>

                            <div>
                              <label
                                class="text-[10px] uppercase font-bold text-muted-foreground mb-1 block">Body</label>
                              <textarea v-model="tab.preflight.body" rows="3"
                                placeholder='{"grant_type": "client_credentials"}'
                                class="w-full bg-background border rounded px-2 py-1 text-sm font-mono" />
                            </div>

                            <div class="grid grid-cols-2 gap-4">
                              <div>
                                <label class="text-[10px] uppercase font-bold text-muted-foreground mb-1 block">Token
                                  Key in Response</label>
                                <input v-model="tab.preflight.tokenKey" placeholder="access_token"
                                  class="w-full bg-background border rounded px-2 py-1 text-sm h-8" />
                              </div>
                              <div>
                                <label class="text-[10px] uppercase font-bold text-muted-foreground mb-1 block">Inject
                                  into Header</label>
                                <input v-model="tab.preflight.tokenHeader" placeholder="Authorization"
                                  class="w-full bg-background border rounded px-2 py-1 text-sm h-8" />
                              </div>
                            </div>
                          </div>
                        </div>
                      </TabsContent>
                    </div>
                  </Tabs>
                </div>
              </ResizablePanel>

              <ResizableHandle with-handle />

              <ResizablePanel :default-size="50" :min-size="20">
                <ResponseViewer :response="tab.response" :url="tab.url" :variables="tabVariablesMap[tab.id] ?? {}"
                  :environment-name="getEnvName(tab)" />
              </ResizablePanel>
            </ResizablePanelGroup>
          </div>

          <!-- Settings View -->
          <div v-else-if="tab.type === 'settings'" class="h-full overflow-auto">
            <ServiceSettingsView :tab="tab" :git-status="gitStatuses?.[tab.serviceId]" :label="label" />
          </div>
        </TabsContent>
      </div>
    </Tabs>

    <div
      v-if="tabs.length === 0"
      class="flex-1 flex flex-col items-center justify-center text-muted-foreground p-6 text-center"
    >
      <div class="p-8 rounded-full bg-muted/20 mb-4">
        <Play class="h-12 w-12 opacity-20" />
      </div>
      <h3 class="text-lg font-medium mb-1">No tabs open</h3>
      <p class="text-sm max-w-xs text-balance mb-6">
        Select an endpoint from the sidebar or choose an action below to get
        started.
      </p>

      <div class="flex items-center gap-3">
        <button
          @click="openServiceDialog"
          class="px-4 py-2 bg-secondary text-secondary-foreground rounded-md text-sm font-medium hover:bg-secondary/80 transition-colors border shadow-sm"
        >
          New Service
        </button>
        <button
          @click="handleNewRequest"
          class="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:bg-primary/90 transition-colors shadow-sm"
        >
          New Endpoint
        </button>
      </div>
    </div>
  </div>
</template>
