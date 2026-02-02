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
import { Loader2, CheckCircle2, XCircle, AlertTriangle } from "lucide-vue-next";
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

watch(
    () => props.open,
    (val) => {
        if (val) {
            runTest();
        }
    }
);
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
                <!-- Loading State -->
                <div v-if="isLoading" class="flex flex-col items-center justify-center py-12 text-muted-foreground">
                    <Loader2 class="h-10 w-10 animate-spin mb-4 text-primary" />
                    <p>Executing pre-flight request...</p>
                </div>

                <!-- Error State (Command Failure) -->
                <div v-else-if="error"
                    class="bg-destructive/10 text-destructive p-4 rounded-md flex items-center gap-3">
                    <XCircle class="h-5 w-5 shrink-0" />
                    <div>
                        <h4 class="font-bold">Test Failed to Execute</h4>
                        <p class="text-sm opacity-90">{{ error }}</p>
                    </div>
                </div>

                <!-- Result State -->
                <div v-else-if="result" class="space-y-6">
                    <!-- Status Banner -->
                    <div :class="[
                        'p-4 rounded-md border flex items-start gap-3',
                        result.success
                            ? 'bg-green-500/10 border-green-500/20 text-green-700 dark:text-green-400'
                            : 'bg-red-500/10 border-red-500/20 text-red-700 dark:text-red-400',
                    ]">
                        <component :is="result.success ? CheckCircle2 : AlertTriangle"
                            class="h-5 w-5 shrink-0 mt-0.5" />
                        <div>
                            <h4 class="font-bold">
                                {{
                                    result.success
                                        ? "Authentication Successful"
                                        : "Authentication Failed"
                                }}
                            </h4>
                            <p class="text-sm opacity-90">
                                {{
                                    result.success
                                        ? "Token extracted successfully and cached."
                                        : result.error ||
                                        "Failed to extract token."
                                }}
                            </p>
                            <div v-if="result.success && result.token"
                                class="mt-2 bg-background/50 p-2 rounded font-mono text-xs break-all border border-green-500/20">
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

                        <TabsContent value="summary" class="space-y-4 pt-4">
                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-1">
                                    <span class="text-xs text-muted-foreground uppercase font-bold">Request URL</span>
                                    <div class="font-mono text-sm break-all p-2 bg-muted rounded">
                                        {{ result.requestUrl }}
                                    </div>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-muted-foreground uppercase font-bold">Method</span>
                                    <div class="font-mono text-sm p-2 bg-muted rounded">
                                        {{ result.requestMethod }}
                                    </div>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-muted-foreground uppercase font-bold">Status</span>
                                    <div class="font-mono text-sm p-2 bg-muted rounded flex items-center gap-2">
                                        <span :class="{
                                            'text-green-600':
                                                result.responseStatus < 300,
                                            'text-red-600':
                                                result.responseStatus >=
                                                400,
                                        }">
                                            {{ result.responseStatus }}
                                        </span>
                                    </div>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-xs text-muted-foreground uppercase font-bold">Time</span>
                                    <div class="font-mono text-sm p-2 bg-muted rounded">
                                        {{ result.timeElapsed }}ms
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

            <div class="p-4 border-t bg-muted/10 flex justify-end">
                <Button @click="isOpen = false">Close</Button>
            </div>
        </DialogContent>
    </Dialog>
</template>
