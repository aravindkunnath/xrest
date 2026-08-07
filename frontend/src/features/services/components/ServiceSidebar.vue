<script setup lang="ts">
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import { useGitIntegration } from "@/composables/useGitIntegration";
import { useServicesStore } from "@/stores/services";
import { Endpoint, Service } from "@/types";
import {
    ChevronDown,
    ChevronRight,
    Download,
    Edit,
    FileText,
    Folder,
    GitBranch,
    GitCommit,
    GitPullRequest,
    Plus,
} from "@lucide/vue";
import { computed, ref, watch } from "vue";

const props = defineProps<{
    selectedServiceId?: string;
}>();

const emit = defineEmits<{
    (e: "select-service", serviceId: string): void;
    (e: "select-endpoint", endpoint: Endpoint): void;
    (e: "select-service-settings", service: Service): void;
    (e: "import-curl", service: Service): void;
    (e: "import-swagger"): void;
    (e: "add-endpoint", service: Service): void;
    (e: "endpoint-context", event: MouseEvent, endpoint: Endpoint): void;
    (e: "select-environments", serviceId: string): void;
    (e: "add-service"): void;
}>();

const servicesStore = useServicesStore();
const { gitStatuses, handlePullGit, handlePushGit, handleInitGit, handleCommitGit, fetchGitStatus } = useGitIntegration();

const commitMessage = ref("");
const isCommitting = ref(false);

const performCommit = async () => {
    if (!activeService.value || !commitMessage.value.trim()) return;
    isCommitting.value = true;
    try {
        await handleCommitGit(activeService.value.id, activeService.value.directory, commitMessage.value.trim());
        commitMessage.value = "";
    } catch (e) {
        console.error("Failed to commit:", e);
    } finally {
        isCommitting.value = false;
    }
};

const isGitExpanded = ref(true);
const isDiffExpanded = ref(false);
const isImportPopoverOpen = ref(false);

const activeService = computed(() => {
    if (!servicesStore.services || servicesStore.services.length === 0) return null;
    if (props.selectedServiceId) {
        return servicesStore.services.find((s) => s.id === props.selectedServiceId) || servicesStore.services[0];
    }
    return servicesStore.services[0];
});

watch(
    () => activeService.value,
    (newService) => {
        if (newService) {
            fetchGitStatus(newService.id, newService.directory);
        }
    },
    { immediate: true, deep: true }
);

const currentGitStatus = computed(() => {
    if (!activeService.value) return null;
    return gitStatuses.value[activeService.value.id] || null;
});

const getMethodColor = (method: string) => {
    switch (method?.toUpperCase()) {
        case "GET":
            return "text-emerald-500 font-bold";
        case "POST":
            return "text-amber-500 font-bold";
        case "PUT":
            return "text-sky-500 font-bold";
        case "DELETE":
            return "text-rose-500 font-bold";
        default:
            return "text-muted-foreground font-bold";
    }
};

const handleServiceChange = (event: Event) => {
    const target = event.target as HTMLSelectElement;
    if (target && target.value) {
        emit("select-service", target.value);
    }
};
</script>

<template>
    <div
        class="flex flex-col h-full min-h-0 bg-sidebar border-r border-border select-none w-full text-sidebar-foreground">
        <!-- Service Selector Dropdown Header -->
        <div class="p-3 border-b border-border flex flex-col gap-1.5 shrink-0">
            <div class="flex items-center justify-between">
                <label class="text-[12px] text-primary font-bold uppercase tracking-wider">
                    Service
                </label>
                <button @click="emit('add-service')"
                    class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                    title="Add Service" data-test="add-service-btn">
                    <Plus class="h-3.5 w-3.5" />
                </button>
            </div>
            <div class="relative flex items-center">
                <select :value="activeService?.id || ''" @change="handleServiceChange"
                    class="w-full bg-background border border-input text-foreground rounded-md py-1.5 pl-3 pr-8 text-sm focus:outline-none focus:ring-1 focus:ring-ring appearance-none cursor-pointer font-medium">
                    <option v-for="service in servicesStore.services" :key="service.id" :value="service.id">
                        {{ service.name }}
                    </option>
                </select>
                <ChevronDown class="absolute right-2.5 h-4 w-4 text-muted-foreground pointer-events-none" />
            </div>
        </div>

        <div v-if="activeService" class="flex-1 flex flex-col min-h-0 overflow-hidden">
            <!-- Scrollable Endpoints Area -->
            <div class="flex-1 overflow-y-auto p-2 space-y-4">
                <!-- Endpoints Section -->
                <div class="flex flex-col gap-1">
                    <div
                        class="px-2 py-1 flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider group/endpoints">
                        <Folder class="h-3.5 w-3.5 text-primary" />
                        <span class="font-bold text-primary">Endpoints</span>
                        <div class="ml-auto flex items-center gap-1">
                            <button @click="emit('add-endpoint', activeService)"
                                class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                                title="Add Endpoint">
                                <Plus class="h-3.5 w-3.5" />
                            </button>
                            <Popover v-model:open="isImportPopoverOpen">
                                <PopoverTrigger as-child>
                                    <button
                                        class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                                        title="Import">
                                        <Download class="h-3.5 w-3.5" />
                                    </button>
                                </PopoverTrigger>
                                <PopoverContent class="w-48 p-1 bg-popover border border-border rounded-md shadow-md"
                                    align="end">
                                    <div class="flex flex-col">
                                        <button @click="
                                            isImportPopoverOpen = false;
                                        emit('import-swagger');
                                        "
                                            class="flex w-full items-center gap-2 px-2.5 py-1.5 hover:bg-accent hover:text-accent-foreground rounded-sm text-xs text-left transition-colors text-foreground">
                                            <span>Import from Swagger</span>
                                        </button>
                                        <button @click="
                                            isImportPopoverOpen = false;
                                        emit('import-curl', activeService);
                                        "
                                            class="flex w-full items-center gap-2 px-2.5 py-1.5 hover:bg-accent hover:text-accent-foreground rounded-sm text-xs text-left transition-colors text-foreground">
                                            <span>Import from cURL</span>
                                        </button>
                                    </div>
                                </PopoverContent>
                            </Popover>
                        </div>
                    </div>

                    <div v-if="!activeService.endpoints || activeService.endpoints.length === 0"
                        class="text-xs text-muted-foreground p-3 text-center">
                        No endpoints found
                    </div>

                    <div v-for="endpoint in activeService.endpoints"
                        :key="endpoint.id + '-' + (endpoint.lastVersion || 1)"
                        @click="emit('select-endpoint', endpoint)"
                        @contextmenu.prevent="emit('endpoint-context', $event, endpoint)"
                        class="flex items-center justify-between px-2 py-1.5 rounded-md hover:bg-sidebar-accent cursor-pointer group transition-colors text-xs">
                        <div class="flex items-center gap-2 min-w-0 flex-1 pr-2">
                            <span
                                :class="['text-[10px] shrink-0 uppercase w-10 text-left', getMethodColor(endpoint.method)]">
                                {{ endpoint.method }}
                            </span>
                            <span class="truncate text-foreground/90 group-hover:text-foreground font-medium">
                                {{ endpoint.name }}
                            </span>
                        </div>

                        <!-- Version Badge on the right -->
                        <span
                            class="text-[10px] px-1.5 py-0.5 rounded border border-border bg-muted/50 text-muted-foreground shrink-0">
                            v{{ endpoint.lastVersion || 1 }}
                        </span>
                    </div>
                </div>
            </div>
            <!-- Environments & Settings Links Section -->
            <div class="space-y-1 pb-2 shrink-0">
                <button @click="emit('select-service-settings', activeService)"
                    class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-sidebar-accent text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors">
                    <Edit class="h-4 w-4" />
                    <span class="font-bold">Configuration</span>
                </button>
            </div>
            <!-- Fixed Bottom Section -->
            <div class="shrink-0 p-3 border-t border-border bg-muted/10 flex flex-col min-h-0">
                <!-- Collapsible Git Section -->
                <div @click="isGitExpanded = !isGitExpanded"
                    class="flex items-center justify-between py-1 text-xs font-semibold text-muted-foreground uppercase tracking-wider cursor-pointer hover:text-foreground transition-colors group">
                    <div class="flex items-center gap-1.5">
                        <component :is="isGitExpanded ? ChevronDown : ChevronRight" class="h-3.5 w-3.5" />
                        <GitBranch class="h-3.5 w-3.5 text-foreground" />
                        <span>Git</span>
                    </div>

                    <!-- Actions -->
                    <div v-if="currentGitStatus?.isGit" class="flex items-center gap-1" @click.stop>
                        <button @click="handlePullGit(activeService.id, activeService.directory)"
                            class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                            title="Pull latest">
                            <GitPullRequest class="h-3.5 w-3.5" />
                        </button>
                        <button @click="handlePushGit(activeService.id, activeService.directory)"
                            class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                            title="Push changes">
                            <GitCommit class="h-3.5 w-3.5" />
                        </button>
                    </div>

                    <div v-else-if="activeService" @click.stop>
                        <button @click="handleInitGit(activeService.id, activeService.directory, activeService.gitUrl)"
                            class="p-1 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1 text-[10px] normal-case"
                            title="Initialize Git Repository">
                            <Plus class="h-3.5 w-3.5" />
                            <span>Init</span>
                        </button>
                    </div>
                </div>


                <div v-show="isGitExpanded" class="space-y-2">
                    <div v-if="currentGitStatus?.isGit" class="text-xs space-y-1">
                        <div class="flex items-center justify-between text-muted-foreground">
                            <span>Branch:</span>
                            <span class="font-mono text-foreground">{{ currentGitStatus.branch || 'main' }}</span>
                        </div>

                        <div v-if="currentGitStatus.hasUncommittedChanges" class="mt-2 space-y-2">
                            <div @click="isDiffExpanded = !isDiffExpanded"
                                class="flex items-center justify-between cursor-pointer text-amber-500 font-medium py-1">
                                <span class="flex items-center gap-1">
                                    <FileText class="h-3 w-3" />
                                    Uncommitted Changes
                                </span>
                                <component :is="isDiffExpanded ? ChevronDown : ChevronRight" class="h-3 w-3" />
                            </div>

                            <div v-show="isDiffExpanded"
                                class="p-2 rounded bg-muted/40 text-[11px] font-mono text-muted-foreground space-y-1 max-h-24 overflow-y-auto">
                                <template
                                    v-if="currentGitStatus.uncommittedFiles && currentGitStatus.uncommittedFiles.length > 0">
                                    <div v-for="file in currentGitStatus.uncommittedFiles" :key="file.path" :class="{
                                        'text-emerald-500': file.status === 'added',
                                        'text-rose-500': file.status === 'deleted',
                                        'text-amber-500': file.status === 'modified',
                                        'text-blue-400': file.status === 'untracked'
                                    }">
                                        {{ file.status === 'added' ? '+' : file.status === 'deleted' ? '-' : file.status
                                        === 'untracked' ? '?' : '~' }}
                                        {{ file.status }} {{ file.path }}
                                    </div>
                                </template>
                                <template v-else>
                                    <div class="italic text-muted-foreground">No modified files detected</div>
                                </template>
                            </div>

                            <!-- Commit Form -->
                            <div class="mt-2 space-y-1.5 p-2 rounded-md bg-muted/30 border border-border/50">
                                <input v-model="commitMessage" type="text" placeholder="Commit message..."
                                    class="w-full text-[11px] px-2 py-1 rounded border border-input bg-background/50 text-foreground placeholder:text-muted-foreground focus:outline-hidden focus:ring-1 focus:ring-ring"
                                    @keyup.enter="performCommit" :disabled="isCommitting" />
                                <button @click="performCommit" :disabled="!commitMessage.trim() || isCommitting"
                                    class="w-full text-[10px] py-1 px-2 bg-emerald-600 text-white rounded font-medium hover:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors">
                                    {{ isCommitting ? 'Committing...' : 'Commit Changes' }}
                                </button>
                            </div>
                        </div>
                        <div v-else class="text-[11px] text-muted-foreground italic py-1">
                            Working directory clean
                        </div>
                    </div>
                    <div v-else
                        class="text-xs text-muted-foreground py-2 text-center bg-muted/20 rounded border border-dashed border-border">
                        No Git repository associated with this service directory.
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
