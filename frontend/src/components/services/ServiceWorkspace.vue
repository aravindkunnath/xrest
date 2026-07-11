<script setup lang="ts">
import {
    ResizableHandle,
    ResizablePanel,
    ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Play, Plus, Settings2, X } from "@lucide/vue";
import { computed } from "vue";

// Components
import RequestBody from "@/components/RequestBody.vue";
import RequestHistory from "@/components/RequestHistory.vue";
import RequestParameters from "@/components/RequestParameters.vue";
import RequestUrlBar from "@/components/RequestUrlBar.vue";
import ResponseViewer from "@/components/ResponseViewer.vue";
import ServiceSettingsView from "@/components/ServiceSettingsView.vue";

// Composables & Utils
import { useDialogState } from "@/composables/useDialogState";
import { useEnvironmentVariables } from "@/composables/useEnvironmentVariables";
import { useRequestExecution } from "@/composables/useRequestExecution";
import { useGitIntegration } from "@/composables/useGitIntegration";
import { useTabManager } from "@/composables/useTabManager";
import { useServicesStore } from "@/stores/services";

const emit = defineEmits<{
    (e: "sync-git", serviceId: string, directory: string): void;
    (e: "init-git", serviceId: string, directory: string, url?: string): void;
    (e: "share-request", tab: any): void;
    // Emitting save intent up? Or logic here? Logic here is mostly request specific.
}>();

const { tabs, activeTab, addTab, closeTab, updateTabSnapshot } =
    useTabManager();

const { gitStatuses } = useGitIntegration();

const { activeEnvironments, getTabVariables, isUnsafeEnv, getEnvName } =
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

const { isUnsafeDialogOpen } = useDialogState();

const { isSending, handleSendRequest } =
    useRequestExecution(isUnsafeDialogOpen);

const servicesStore = useServicesStore();

// Logic for saving and restoring that was in ServicesView

import { toast } from "vue-sonner";
const handleSaveRequest = async (tab: any) => {
    if (!tab.endpointId) {
        toast.error("Cannot save: This tab is not linked to an endpoint");
        return;
    }

    // Find the service and endpoint
    let serviceIndex = -1;
    let endpointIndex = -1;

    for (let i = 0; i < servicesStore.services.length; i++) {
        const idx = servicesStore.services[i].endpoints.findIndex(
            (e) => e.id === tab.endpointId,
        );
        if (idx !== -1) {
            serviceIndex = i;
            endpointIndex = idx;
            break;
        }
    }

    if (serviceIndex === -1 || endpointIndex === -1) {
        toast.error("Endpoint not found in any service");
        return;
    }

    const service = servicesStore.services[serviceIndex];
    const endpoint = service.endpoints[endpointIndex];

    // Update endpoint details
    let newPath = tab.url;
    if (service.environments.length > 0) {
        for (const envGroup of service.environments) {
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
    if (
        !newPath.startsWith("/") &&
        !newPath.includes("://") &&
        !newPath.startsWith("{{")
    ) {
        newPath = "/" + newPath;
    }

    // Consolidate preflight structure
    const preflight = {
        enabled: tab.preflight.enabled,
        method: tab.preflight.method,
        url: tab.preflight.url,
        body: tab.preflight.body,
        headers: tab.preflight.headers || [],
        cacheToken: tab.preflight.cacheToken,
        cacheDuration: tab.preflight.cacheDuration,
        cacheDurationKey: tab.preflight.cacheDurationKey,
        cacheDurationUnit: tab.preflight.cacheDurationUnit,
        tokenKey: tab.preflight.tokenKey,
        tokenHeader: tab.preflight.tokenHeader,
    };

    const updatedEndpoint = {
        ...endpoint,
        method: tab.method,
        url: newPath,
        params: tab.params
            .filter((p: any) => p.name)
            .map((p: any) => ({
                name: p.name,
                value: p.value,
                enabled: p.enabled ?? true,
            })),
        headers: tab.headers
            .filter((h: any) => h.name)
            .map((h: any) => ({
                name: h.name,
                value: h.value,
                enabled: h.enabled ?? true,
            })),
        body: tab.body.content,
        preflight: preflight,
        metadata: {
            ...endpoint.metadata,
            lastUpdated: Date.now(),
        },
    };

    const updatedService = {
        ...service,
        endpoints: [...service.endpoints],
    };
    updatedService.endpoints[endpointIndex] = updatedEndpoint;

    try {
        await servicesStore.updateService(serviceIndex, updatedService);

        // Sync versions from store after backend update
        const finalService = servicesStore.services[serviceIndex];
        const finalEndpoint = finalService?.endpoints.find(
            (e) => e.id === tab.endpointId,
        );
        if (finalEndpoint) {
            tab.versions = finalEndpoint.versions || [];
        }

        toast.success("Endpoint saved", {
            description: `Changes to "${endpoint.name}" have been persisted.`,
        });

        updateTabSnapshot(tab);
    } catch (error) {
        toast.error("Failed to save endpoint");
    }
};

const restoreVersion = (tab: any, version: any) => {
    tab.method = version.config.method;
    tab.url = version.config.url;
    tab.auth.type = version.config.authType;
    tab.params = version.config.params.map((p: any) => ({
        ...p,
        enabled: true,
    }));
    tab.headers = version.config.headers.map((h: any) => ({
        ...h,
        enabled: true,
    }));
    tab.body.content = version.config.body;
    tab.preflight = JSON.parse(JSON.stringify(version.config.preflight));

    toast.success(`Restored to version ${version.version}`);
};

// Service Settings Handlers - Removed as they are now handled via useServiceSettings in the component
</script>

<template>
    <div class="flex flex-col h-full bg-background">
        <Tabs v-model="activeTab" class="flex-1 flex flex-col overflow-hidden">
            <div class="flex items-center border-b bg-muted/20 px-2 pt-1 gap-1">
                <TabsList class="h-9 bg-transparent p-0 gap-px">
                    <div
                        v-for="tab in tabs"
                        :key="tab.id"
                        class="group relative flex items-center"
                    >
                        <TabsTrigger
                            :value="tab.id"
                            :class="[
                                'h-8 px-3 rounded-t-md rounded-b-none border-b-0 data-[state=active]:bg-background data-[state=active]:border data-[state=active]:border-b-background -mb-px gap-2 relative overflow-hidden',
                                isUnsafeEnv(tab) ? 'bg-destructive/5' : '',
                            ]"
                        >
                            <div
                                v-if="isUnsafeEnv(tab)"
                                class="absolute top-0 left-0 right-0 h-[2px] bg-destructive shadow-[0_0_8px_rgba(239,68,68,0.5)]"
                            ></div>
                            <template v-if="tab.type === 'settings'">
                                <Settings2 class="h-3 w-3 text-primary/70" />
                            </template>
                            <template v-else>
                                <span
                                    :class="[
                                        'font-bold ',
                                        tab.method === 'GET'
                                            ? 'text-green-600'
                                            : tab.method === 'POST'
                                              ? 'text-orange-600'
                                              : tab.method === 'PUT'
                                                ? 'text-blue-600'
                                                : 'text-red-600',
                                    ]"
                                    >{{ tab.method || "GET" }}</span
                                >
                            </template>
                            <span class="max-w-[120px] truncate">{{
                                tab.title
                            }}</span>
                            <div
                                v-if="tab.isEdited"
                                class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse shrink-0"
                            ></div>
                            <button
                                @click.stop="closeTab(tab.id)"
                                class="ml-1 p-0.5 rounded-sm hover:bg-muted opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                                <X class="h-3 w-3" />
                            </button>
                        </TabsTrigger>
                    </div>
                </TabsList>
                <button
                    @click="addTab()"
                    class="p-1.5 hover:bg-muted rounded-md transition-colors text-muted-foreground mb-1"
                >
                    <Plus class="h-4 w-4" />
                </button>
            </div>

            <div class="flex-1 overflow-hidden">
                <TabsContent
                    v-for="tab in tabs as any"
                    :key="tab.id"
                    :value="tab.id"
                    class="h-full mt-0 focus-visible:ring-0"
                >
                    <!-- Request View -->
                    <ResizablePanelGroup
                        v-if="tab.type !== 'settings'"
                        direction="vertical"
                    >
                        <ResizablePanel :default-size="50" :min-size="20">
                            <div class="h-full p-4 overflow-auto">
                                <!-- URL Bar component -->
                                <RequestUrlBar
                                    v-model:method="tab.method"
                                    v-model:url="tab.url"
                                    :is-sending="isSending"
                                    :is-unsafe="isUnsafeEnv(tab)"
                                    :variables="tabVariablesMap[tab.id] ?? {}"
                                    :environment-name="
                                        tab.serviceId
                                            ? activeEnvironments[tab.serviceId]
                                            : ''
                                    "
                                    @send="handleSendRequest(tab)"
                                    @save="handleSaveRequest(tab)"
                                    @share="emit('share-request', tab)"
                                />

                                <div
                                    class="flex gap-4 border-b pb-0 mb-4 font-medium text-muted-foreground"
                                >
                                    <button
                                        @click="tab.activeSubTab = 'params'"
                                        :class="[
                                            'pb-2 -mb-px px-1 transition-colors relative',
                                            tab.activeSubTab === 'params'
                                                ? 'text-primary'
                                                : 'hover:text-foreground',
                                        ]"
                                    >
                                        Params
                                        <div
                                            v-if="tab.activeSubTab === 'params'"
                                            class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary rounded-t-full"
                                        ></div>
                                    </button>
                                    <button
                                        @click="tab.activeSubTab = 'headers'"
                                        :class="[
                                            'pb-2 -mb-px px-1 transition-colors relative',
                                            tab.activeSubTab === 'headers'
                                                ? 'text-primary'
                                                : 'hover:text-foreground',
                                        ]"
                                    >
                                        Headers
                                        <div
                                            v-if="
                                                tab.activeSubTab === 'headers'
                                            "
                                            class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary rounded-t-full"
                                        ></div>
                                    </button>
                                    <button
                                        @click="tab.activeSubTab = 'body'"
                                        :class="[
                                            'pb-2 -mb-px px-1 transition-colors relative',
                                            tab.activeSubTab === 'body'
                                                ? 'text-primary'
                                                : 'hover:text-foreground',
                                        ]"
                                    >
                                        Body
                                        <div
                                            v-if="tab.activeSubTab === 'body'"
                                            class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary rounded-t-full"
                                        ></div>
                                    </button>
                                    <button
                                        @click="tab.activeSubTab = 'versions'"
                                        :class="[
                                            'pb-2 -mb-px px-1 transition-colors relative',
                                            tab.activeSubTab === 'versions'
                                                ? 'text-primary'
                                                : 'hover:text-foreground',
                                        ]"
                                    >
                                        Versions
                                        <div
                                            v-if="
                                                tab.activeSubTab === 'versions'
                                            "
                                            class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary rounded-t-full"
                                        ></div>
                                    </button>
                                </div>

                                <!-- Sub-tabs Content -->
                                <div class="min-h-[200px]">
                                    <!-- Params and Headers Table -->
                                    <RequestParameters
                                        v-if="tab.activeSubTab === 'params'"
                                        v-model:items="tab.params"
                                        title="Query Parameters"
                                        :variables="
                                            tabVariablesMap[tab.id] ?? {}
                                        "
                                        :environment-name="getEnvName(tab)"
                                    />
                                    <RequestParameters
                                        v-else-if="
                                            tab.activeSubTab === 'headers'
                                        "
                                        v-model:items="tab.headers"
                                        title="Request Headers"
                                        :variables="
                                            tabVariablesMap[tab.id] ?? {}
                                        "
                                        :environment-name="getEnvName(tab)"
                                    />

                                    <RequestBody
                                        v-else-if="tab.activeSubTab === 'body'"
                                        v-model:body="tab.body"
                                        :variables="
                                            tabVariablesMap[tab.id] ?? {}
                                        "
                                        :environment-name="getEnvName(tab)"
                                    />

                                    <RequestHistory
                                        v-else-if="
                                            tab.activeSubTab === 'versions'
                                        "
                                        :tabId="tab.id"
                                        :versions="tab.versions || []"
                                        @restore="(v) => restoreVersion(tab, v)"
                                    />
                                </div>
                            </div>
                        </ResizablePanel>

                        <ResizableHandle with-handle />

                        <ResizablePanel :default-size="50" :min-size="20">
                            <ResponseViewer
                                v-if="tab.response"
                                :response="tab.response"
                                :url="tab.url"
                                :variables="tabVariablesMap[tab.id] ?? {}"
                                :environment-name="
                                    tab.serviceId
                                        ? activeEnvironments[tab.serviceId]
                                        : ''
                                "
                            />
                            <div
                                v-else
                                class="h-full flex flex-col items-center justify-center text-muted-foreground gap-3 bg-muted/5"
                            >
                                <div
                                    class="p-4 rounded-full bg-muted/20 border border-dashed border-muted"
                                >
                                    <Play class="h-8 w-8 opacity-20" />
                                </div>
                                <p class="font-medium tracking-tight">
                                    Send a request to see the response
                                </p>
                            </div>
                        </ResizablePanel>
                    </ResizablePanelGroup>

                    <!-- Service Settings View -->
                    <ServiceSettingsView
                        v-else
                        :tab="tab"
                        :git-status="gitStatuses[tab.serviceId]"
                    />
                </TabsContent>
            </div>
        </Tabs>
    </div>
</template>
