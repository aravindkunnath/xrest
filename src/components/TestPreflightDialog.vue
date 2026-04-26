<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Loader2, CheckCircle2, XCircle, AlertTriangle, Play, RefreshCw } from "lucide-vue-next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const props = defineProps<{
    open: boolean;
    serviceId: string;
    config: any;
    variables: Record<string, string>;
}>();

const emit = defineEmits(["update:open"]);

const isOpen = computed({
    get: () => props.open,
    set: (val) => emit("update:open", val),
});

const isLoading = ref(false);
const result = ref<any>(null);
const error = ref<string | null>(null);
const activeTab = ref("summary");

const runTest = async () => {
    if (!props.serviceId) {
        error.value = "Service ID is missing. Please save the service first.";
        return;
    }

    isLoading.value = true;
    error.value = null;
    result.value = null;
    activeTab.value = "summary";

    try {
        const res = await invoke("test_preflight_config", {
            serviceId: props.serviceId,
            config: JSON.parse(JSON.stringify(props.config)),
            variables: props.variables,
        });
        result.value = res;
    } catch (err: any) {
        error.value = err?.toString() || "Unknown error occurred";
    } finally {
        isLoading.value = false;
    }
};

</script>

<template>
    <Dialog v-model:open="isOpen">
        <DialogContent class="max-w-4xl max-h-[85vh] flex flex-col p-0 gap-0 overflow-hidden">
            <div class="p-6 border-b">
                <DialogHeader>
                    <DialogTitle>Pre-flight Sequence Test</DialogTitle>
                    <DialogDescription>Testing your authentication flow
                        configuration.</DialogDescription>
                </DialogHeader>
            </div>

            <div class="flex-1 overflow-y-auto p-6">
                <!-- Initial/Idle State -->
                <div v-if="!isLoading && !result && !error" class="flex flex-col items-center justify-center py-12 text-muted-foreground gap-4">
                    <div class="h-20 w-20 bg-muted/20 rounded-full flex items-center justify-center border border-muted-foreground/10">
                        <Play class="h-8 w-8 text-muted-foreground/40" />
                    </div>
                    <div class="text-center">
                        <h4 class="font-bold text-muted-foreground uppercase text-xs tracking-widest mb-1">Ready to Test</h4>
                        <p class="text-xs text-muted-foreground/60 max-w-[250px]">
                            Click the button below to execute the pre-flight sequence and verify your configuration.
                        </p>
                    </div>
                    <Button @click="runTest" class="gap-2 px-8">
                        <Play class="h-4 w-4 fill-current" /> START TEST
                    </Button>
                </div>

                <!-- Loading State -->
                <div v-else-if="isLoading" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
                    <Loader2 class="h-10 w-10 animate-spin mb-4 text-primary" />
                    <p class="text-sm font-medium">Executing pre-flight request...</p>
                </div>

                <!-- Error State (Command Failure) -->
                <div v-else-if="error"
                    class="bg-destructive/10 text-destructive p-6 rounded-lg flex flex-col items-center gap-4 text-center">
                    <XCircle class="h-10 w-10 shrink-0" />
                    <div>
                        <h4 class="font-bold">Test Failed to Execute</h4>
                        <p class="text-sm opacity-90 font-mono mt-1">{{ error }}</p>
                    </div>
                    <Button variant="outline" @click="runTest" class="gap-2">
                        <RefreshCw class="h-4 w-4" /> RETRY TEST
                    </Button>
                </div>

                <!-- Result State -->
                <div v-else-if="result" class="space-y-6">
                    <!-- Status Banner -->
                    <div :class="[
                        'p-5 rounded-xl border flex items-start gap-4 transition-all duration-300',
                        result.success
                            ? 'bg-green-500/10 border-green-500/20 text-green-700 dark:text-green-400 shadow-[0_0_15px_rgba(34,197,94,0.05)]'
                            : 'bg-red-500/10 border-red-500/20 text-red-700 dark:text-red-400 shadow-[0_0_15px_rgba(239,68,68,0.05)]',
                    ]">
                        <div :class="[
                            'p-2 rounded-lg shrink-0',
                            result.success ? 'bg-green-500/20' : 'bg-red-500/20'
                        ]">
                            <component :is="result.success ? CheckCircle2 : AlertTriangle" class="h-5 w-5" />
                        </div>
                        <div class="flex-1 min-w-0">
                            <h4 class="font-bold text-sm tracking-tight mb-0.5">
                                {{
                                    result.success
                                        ? "Authentication Successful"
                                        : "Authentication Failed"
                                }}
                            </h4>
                            <p class="text-[11px] opacity-80 leading-relaxed">
                                {{
                                    result.success
                                        ? "The token was successfully extracted and is now active in the cache."
                                        : result.error || "The pre-flight sequence failed to return a valid token."
                                }}
                            </p>
                            <div v-if="result.success && result.token"
                                class="mt-3 bg-background/40 p-3 rounded-lg font-mono text-[10px] break-all border border-green-500/10 shadow-inner">
                                <span class="text-muted-foreground mr-2 select-none font-bold uppercase tracking-widest text-[9px]">Token:</span>
                                {{ result.token }}
                            </div>
                        </div>
                    </div>

                    <Tabs v-model="activeTab" class="w-full">
                        <TabsList class="grid w-full grid-cols-3">
                            <TabsTrigger value="summary">Summary</TabsTrigger>
                            <TabsTrigger value="request">Request</TabsTrigger>
                            <TabsTrigger value="response">Response</TabsTrigger>
                        </TabsList>

                        <TabsContent value="summary" class="space-y-6 pt-4">
                            <div class="space-y-4">
                                <div class="space-y-1.5">
                                    <span class="text-[10px] text-muted-foreground uppercase font-bold tracking-tight ml-1">Request URL</span>
                                    <div class="font-mono text-xs break-all p-3 bg-muted/30 border rounded-lg shadow-inner min-h-[40px] flex items-center">
                                        {{ result.requestUrl || 'None' }}
                                    </div>
                                </div>
                                
                                <div class="grid grid-cols-3 gap-4">
                                    <div class="space-y-1.5">
                                        <span class="text-[10px] text-muted-foreground uppercase font-bold tracking-tight ml-1">Method</span>
                                        <div class="font-bold text-xs p-2.5 bg-muted/30 border rounded-lg flex items-center justify-center">
                                            {{ result.requestMethod }}
                                        </div>
                                    </div>
                                    <div class="space-y-1.5">
                                        <span class="text-[10px] text-muted-foreground uppercase font-bold tracking-tight ml-1">Status</span>
                                        <div class="p-2.5 bg-muted/30 border rounded-lg flex items-center justify-center font-bold text-xs">
                                            <span :class="[
                                                result.responseStatus > 0 && result.responseStatus < 300 ? 'text-green-500' :
                                                result.responseStatus >= 400 ? 'text-red-500' :
                                                'text-muted-foreground'
                                            ]">
                                                {{ result.responseStatus || 'N/A' }}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="space-y-1.5">
                                        <span class="text-[10px] text-muted-foreground uppercase font-bold tracking-tight ml-1">Duration</span>
                                        <div class="font-mono text-xs p-2.5 bg-muted/30 border rounded-lg flex items-center justify-center">
                                            {{ result.timeElapsed }}ms
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <!-- Telemetry Section -->
                            <div class="pt-6 border-t mt-4">
                                <div class="flex items-center gap-2 mb-4">
                                    <h4 class="text-[11px] font-bold uppercase text-muted-foreground tracking-widest">Cache & Extraction Info</h4>
                                    <div class="h-px flex-1 bg-border/50"></div>
                                </div>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div class="bg-muted/20 p-4 rounded-xl border flex flex-col gap-3">
                                        <span class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight">Cache Status</span>
                                        <div class="flex flex-col gap-2">
                                            <div class="flex items-center gap-2">
                                                <span :class="[
                                                    'px-2.5 py-1 rounded-full text-[10px] font-extrabold uppercase border shadow-sm',
                                                    result.cacheStatus === 'hit' ? 'bg-blue-500/10 border-blue-500/30 text-blue-500' :
                                                    result.cacheStatus === 'miss' ? 'bg-orange-500/10 border-orange-500/30 text-orange-500' :
                                                    result.cacheStatus === 'error' ? 'bg-red-500/10 border-red-500/30 text-red-500' :
                                                    'bg-muted border-muted text-muted-foreground'
                                                ]">
                                                    {{ result.cacheStatus || 'None' }}
                                                </span>
                                            </div>
                                            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                                                {{ result.cacheStatusDetail || 'No cache telemetry available.' }}
                                            </p>
                                        </div>
                                    </div>
                                    <div class="bg-muted/20 p-4 rounded-xl border flex flex-col gap-3">
                                        <span class="text-[10px] uppercase font-bold text-muted-foreground tracking-tight">Extraction Path</span>
                                        <div class="flex flex-col gap-2">
                                            <div class="font-mono text-xs p-2 bg-background/50 rounded border border-dashed text-center min-h-[32px] flex items-center justify-center">
                                                {{ result.extractionPath || 'None' }}
                                            </div>
                                            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                                                The JSON path used to find the token in the response.
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </TabsContent>

                        <TabsContent value="request" class="space-y-4 pt-4">
                            <div class="space-y-2">
                                <h4 class="text-sm font-medium">Headers</h4>
                                <div v-if="result.requestHeaders.length > 0" class="border rounded text-xs font-mono">
                                    <div v-for="h in result.requestHeaders" :key="h.name"
                                        class="flex border-b last:border-0 p-2">
                                        <span class="w-1/3 text-muted-foreground border-r pr-2 mr-2 truncate">{{ h.name
                                            }}</span>
                                        <span class="flex-1 break-all">{{
                                            h.value
                                            }}</span>
                                    </div>
                                </div>
                                <div v-else class="text-muted-foreground text-xs italic">
                                    No headers
                                </div>
                            </div>
                            <div class="space-y-2">
                                <h4 class="text-sm font-medium">Body</h4>
                                <div class="bg-muted p-3 rounded-md overflow-x-auto">
                                    <pre class="text-xs font-mono">{{
                                        result.requestBody || "(Empty)"
                                    }}</pre>
                                </div>
                            </div>
                        </TabsContent>

                        <TabsContent value="response" class="space-y-4 pt-4">
                            <div class="space-y-2">
                                <h4 class="text-sm font-medium">Headers</h4>
                                <div v-if="result.responseHeaders.length > 0" class="border rounded text-xs font-mono">
                                    <div v-for="h in result.responseHeaders" :key="h.name"
                                        class="flex border-b last:border-0 p-2">
                                        <span class="w-1/3 text-muted-foreground border-r pr-2 mr-2 truncate">{{ h.name
                                            }}</span>
                                        <span class="flex-1 break-all">{{
                                            h.value
                                            }}</span>
                                    </div>
                                </div>
                                <div v-else class="text-muted-foreground text-xs italic">
                                    No headers
                                </div>
                            </div>
                            <div class="space-y-2">
                                <h4 class="text-sm font-medium">Body</h4>
                                <div class="bg-muted p-3 rounded-md overflow-x-auto max-h-[300px]">
                                    <pre class="text-xs font-mono">{{
                                        result.responseBody || "(Empty)"
                                    }}</pre>
                                </div>
                            </div>
                        </TabsContent>
                    </Tabs>
                </div>
            </div>

            <div class="p-4 border-t bg-muted/5 flex justify-end gap-3">
                <Button variant="ghost" @click="runTest" :disabled="isLoading" class="text-xs h-9">
                    <RefreshCw class="h-3.5 w-3.5 mr-2" :class="{ 'animate-spin': isLoading }" />
                    Re-run Test
                </Button>
                <Button @click="isOpen = false" class="text-xs h-9 px-6 font-bold">Close Dialog</Button>
            </div>
        </DialogContent>
    </Dialog>
</template>
