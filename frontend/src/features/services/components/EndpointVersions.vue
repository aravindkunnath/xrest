<script setup lang="ts">
import { useVersionsStore } from "@/stores/versions";
import { useServicesStore } from "@/stores/services";
import { useTabsStore } from "@/stores/tabs";
import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
} from "@/components/ui/accordion";
import { Button } from "@/components/ui/button";
import {
    AlertCircle,
    ArrowRight,
    ArrowUpRight,
    Layers,
    Minus,
    Plus,
    RefreshCw,
} from "@lucide/vue";
import { computed, onMounted, watch } from "vue";

const versionStore = useVersionsStore();
const servicesStore = useServicesStore();
const tabsStore = useTabsStore();

const groups = computed(() =>
    servicesStore.services
        .map((service) => ({
            service,
            endpoints: service.endpoints.map((endpoint) => ({
                endpoint,
                versions: versionStore.getEntries(endpoint.id),
                count: versionStore.getCount(endpoint.id),
            })),
        }))
        .filter((group) => group.endpoints.some((e) => e.count > 0)),
);

const loadAll = () => {
    servicesStore.services.forEach((service) => {
        service.endpoints.forEach((endpoint) => {
            versionStore.loadVersions(service.id, endpoint.id, 50);
        });
    });
};

onMounted(loadAll);
watch(() => servicesStore.services, loadAll);

const sortedVersions = (entries: any[]) =>
    [...entries].sort((a, b) => {
        const verA =
            typeof a.version === "number" ? a.version : parseInt(a.version) || 0;
        const verB =
            typeof b.version === "number" ? b.version : parseInt(b.version) || 0;
        return verB - verA;
    });

const hasPrevious = (versions: any[], index: number) =>
    sortedVersions(versions)[index + 1] != null;

const handleRestore = (v: any, endpointId: string) => {
    const t = tabsStore.tabs.find(
        (t) => t.id === `endpoint-${endpointId}`,
    );
    if (!t) return;
    t.method = v.config.method;
    t.url = v.config.url;
    t.params = JSON.parse(JSON.stringify(v.config.params));
    t.headers = JSON.parse(JSON.stringify(v.config.headers));
    t.body.content = v.config.body;
    t.preflight = JSON.parse(JSON.stringify(v.config.preflight));
};

const getMethodColor = (method: string) => {
    switch (method?.toUpperCase()) {
        case "GET":
            return "text-green-600 dark:text-green-400";
        case "POST":
            return "text-orange-600 dark:text-orange-400";
        case "PUT":
            return "text-blue-600 dark:text-blue-400";
        case "DELETE":
            return "text-red-600 dark:text-red-400";
        case "PATCH":
            return "text-purple-600 dark:text-purple-400";
        default:
            return "text-muted-foreground";
    }
};

const diffArrays = (prev: any[] = [], curr: any[] = []) => {
    const changes: any[] = [];
    const prevMap = new Map((prev || []).map((i) => [i.name, i]));
    const currMap = new Map((curr || []).map((i) => [i.name, i]));

    (curr || []).forEach((item) => {
        const p = prevMap.get(item.name);
        if (!p) {
            changes.push({ type: "added", name: item.name, value: item.value });
        } else if (p.value !== item.value || p.enabled !== item.enabled) {
            changes.push({
                type: "changed",
                name: item.name,
                oldValue: p.value,
                newValue: item.value,
                oldEnabled: p.enabled,
                newEnabled: item.enabled,
            });
        }
    });

    (prev || []).forEach((item) => {
        if (!currMap.has(item.name)) {
            changes.push({
                type: "removed",
                name: item.name,
                value: item.value,
            });
        }
    });

    return changes;
};

const getVersionChanges = (sorted: any[], current: any, index: number) => {
    const previous = sorted[index + 1];
    if (!previous) return null;

    const changes: any = {
        method:
            current.config.method !== previous.config.method
                ? { old: previous.config.method, new: current.config.method }
                : null,
        url:
            current.config.url !== previous.config.url
                ? { old: previous.config.url, new: current.config.url }
                : null,
        headers: diffArrays(previous.config.headers, current.config.headers),
        params: diffArrays(previous.config.params, current.config.params),
        body:
            current.config.body !== previous.config.body
                ? { old: previous.config.body, new: current.config.body }
                : null,
        preflight: diffPreflight(
            previous.config.preflight,
            current.config.preflight,
        ),
    };

    const hasChanges =
        changes.method ||
        changes.url ||
        changes.headers.length > 0 ||
        changes.params.length > 0 ||
        changes.body ||
        changes.preflight;

    return hasChanges ? changes : null;
};

const diffPreflight = (prev: any = {}, curr: any = {}) => {
    const diffs: any = {};
    let hasChanges = false;

    const keys = [
        "enabled",
        "method",
        "url",
        "body",
        "tokenKey",
        "tokenHeader",
    ];
    keys.forEach((key) => {
        if ((prev?.[key] ?? null) !== (curr?.[key] ?? null)) {
            diffs[key] = {
                old: prev?.[key] ?? "(none)",
                new: curr?.[key] ?? "(none)",
            };
            hasChanges = true;
        }
    });

    return hasChanges ? diffs : null;
};

const vc = (endpointData: any, v: any, index: number) =>
    getVersionChanges(
        sortedVersions(endpointData.versions),
        v,
        index,
    );
</script>

<template>
    <div class="space-y-2">
        <template v-if="groups.length > 0">
            <Accordion type="multiple" class="w-full space-y-2">
                <AccordionItem
                    v-for="group in groups"
                    :key="group.service.id"
                    :value="group.service.id"
                    class="border rounded-md bg-card"
                >
                    <AccordionTrigger class="px-4">
                        <span class="flex items-center gap-3">
                            <span class="font-semibold">{{
                                group.service.name
                            }}</span>
                            <span
                                class="text-muted-foreground bg-muted px-2 py-0.5 rounded-full text-xs font-bold"
                            >
                                {{ group.endpoints.length }} endpoints
                            </span>
                        </span>
                    </AccordionTrigger>
                    <AccordionContent>
                        <Accordion type="multiple" class="w-full">
                            <AccordionItem
                                v-for="endpointData in group.endpoints"
                                :key="endpointData.endpoint.id"
                                :value="endpointData.endpoint.id"
                                class="border-b last:border-b-0"
                            >
                                <AccordionTrigger class="px-4 py-3">
                                    <span
                                        class="flex items-center gap-3 w-full text-left"
                                    >
                                        <span
                                            :class="[
                                                'text-xs font-bold px-1 py-0.5 rounded bg-muted/50',
                                                getMethodColor(
                                                    endpointData.endpoint
                                                        .method,
                                                ),
                                            ]"
                                        >
                                            {{ endpointData.endpoint.method }}
                                        </span>
                                        <span
                                            class="truncate text-sm font-medium"
                                        >
                                            {{ endpointData.endpoint.name }}
                                        </span>
                                        <span
                                            class="text-muted-foreground bg-muted px-2 py-0.5 rounded-full text-xs font-bold ml-auto"
                                        >
                                            {{ endpointData.count }} Versions
                                        </span>
                                    </span>
                                </AccordionTrigger>
                                <AccordionContent class="pb-4">
                                    <div
                                        v-if="endpointData.versions.length === 0"
                                        class="flex flex-col items-center justify-center py-8 text-muted-foreground space-y-2"
                                    >
                                        <AlertCircle
                                            class="h-8 w-8 opacity-20"
                                        />
                                        <p class="text-sm">
                                            No version history available for this
                                            endpoint yet.
                                        </p>
                                    </div>
                                    <Accordion
                                        v-else
                                        type="single"
                                        collapsible
                                        class="w-full px-4"
                                    >
                                        <AccordionItem
                                            v-for="(v, index) in sortedVersions(
                                                endpointData.versions,
                                            )"
                                            :key="v.version"
                                            :value="v.version.toString()"
                                            class="border rounded-md px-4 mb-2 bg-card overflow-hidden"
                                        >
                                            <AccordionTrigger
                                                class="hover:no-underline py-3"
                                            >
                                                <div
                                                    class="flex items-center gap-4 w-full text-left"
                                                >
                                                    <div class="flex flex-col">
                                                        <span
                                                            class="text-sm font-bold text-muted-foreground uppercase opacity-70"
                                                        >
                                                            v{{ v.version }}
                                                        </span>
                                                    </div>
                                                    <div
                                                        class="flex flex-col flex-1 min-w-0"
                                                    >
                                                        <div
                                                            class="flex items-center gap-2"
                                                        >
                                                            <span
                                                                :class="[
                                                                    'text-xs font-bold px-1 py-0.5 rounded bg-muted/50',
                                                                    getMethodColor(
                                                                        v.config
                                                                            .method,
                                                                    ),
                                                                ]"
                                                            >
                                                                {{
                                                                    v.config
                                                                        .method
                                                                }}
                                                            </span>
                                                            <span
                                                                class="truncate text-sm font-medium"
                                                                :title="
                                                                    v.config.url
                                                                "
                                                            >
                                                                {{
                                                                    v.config.url
                                                                }}
                                                            </span>
                                                        </div>
                                                        <span
                                                            class="text-xs text-muted-foreground"
                                                        >
                                                            {{
                                                                v.lastUpdated
                                                                    ? new Date(
                                                                          v.lastUpdated *
                                                                              1000,
                                                                      ).toLocaleString()
                                                                    : "Unknown"
                                                            }}
                                                        </span>
                                                    </div>
                                                </div>
                                            </AccordionTrigger>
                                            <AccordionContent class="pb-4">
                                                <div
                                                    class="pt-2 space-y-4 border-t"
                                                >
                                                    <div
                                                        class="flex justify-end pb-2"
                                                    >
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            class="h-8 px-3 gap-1.5 text-sm font-bold hover:text-primary hover:border-primary/50 transition-colors"
                                                            @click="
                                                                handleRestore(
                                                                    v,
                                                                    endpointData
                                                                        .endpoint
                                                                        .id,
                                                                )
                                                            "
                                                        >
                                                            <RefreshCw
                                                                class="h-3.5 w-3.5"
                                                            />
                                                            RESTORE THIS VERSION
                                                        </Button>
                                                    </div>

                                                    <template
                                                        v-if="
                                                            hasPrevious(
                                                                endpointData.versions,
                                                                index,
                                                            )
                                                        "
                                                    >
                                                        <div
                                                            v-if="
                                                                !vc(
                                                                    endpointData,
                                                                    v,
                                                                    index,
                                                                )
                                                            "
                                                            class="text-sm text-muted-foreground italic bg-muted/30 p-2 rounded border border-dashed text-center"
                                                        >
                                                            No measurable changes
                                                            in this version
                                                            compared to the
                                                            previous one.
                                                        </div>
                                                        <div
                                                            v-else
                                                            class="space-y-4"
                                                        >
                                                            <div
                                                                v-if="
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).method ||
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).url
                                                                "
                                                                class="space-y-2"
                                                            >
                                                                <h4
                                                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                                                                >
                                                                    <ArrowUpRight
                                                                        class="h-3 w-3"
                                                                    />
                                                                    General Details
                                                                </h4>
                                                                <div
                                                                    class="space-y-1"
                                                                >
                                                                    <div
                                                                        v-if="
                                                                            vc(
                                                                                endpointData,
                                                                                v,
                                                                                index,
                                                                            ).method
                                                                        "
                                                                        class="flex items-center gap-2 text-sm"
                                                                    >
                                                                        <span
                                                                            class="w-16 text-muted-foreground"
                                                                            >Method:</span
                                                                        >
                                                                        <span
                                                                            class="line-through opacity-50"
                                                                            >{{
                                                                                vc(
                                                                                    endpointData,
                                                                                    v,
                                                                                    index,
                                                                                ).method
                                                                                    .old
                                                                            }}</span
                                                                        >
                                                                        <ArrowRight
                                                                            class="h-3 w-3 text-muted-foreground"
                                                                        />
                                                                        <span
                                                                            :class="
                                                                                getMethodColor(
                                                                                    vc(
                                                                                        endpointData,
                                                                                        v,
                                                                                        index,
                                                                                    ).method
                                                                                        .new,
                                                                                )
                                                                            "
                                                                            class="font-bold"
                                                                            >{{
                                                                                vc(
                                                                                    endpointData,
                                                                                    v,
                                                                                    index,
                                                                                ).method
                                                                                    .new
                                                                            }}</span
                                                                        >
                                                                    </div>
                                                                    <div
                                                                        v-if="
                                                                            vc(
                                                                                endpointData,
                                                                                v,
                                                                                index,
                                                                            ).url
                                                                        "
                                                                        class="flex flex-col gap-1 text-sm"
                                                                    >
                                                                        <span
                                                                            class="text-muted-foreground"
                                                                            >URL:</span
                                                                        >
                                                                        <div
                                                                            class="pl-2 border-l-2 border-primary/20 break-all space-y-1"
                                                                        >
                                                                            <div
                                                                                class="line-through opacity-50"
                                                                            >
                                                                                {{
                                                                                    vc(
                                                                                        endpointData,
                                                                                        v,
                                                                                        index,
                                                                                    ).url
                                                                                        .old
                                                                                }}
                                                                            </div>
                                                                            <div
                                                                                class="text-primary font-medium"
                                                                            >
                                                                                {{
                                                                                    vc(
                                                                                        endpointData,
                                                                                        v,
                                                                                        index,
                                                                                    ).url
                                                                                        .new
                                                                                }}
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div
                                                                v-if="
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).headers
                                                                        .length > 0
                                                                "
                                                                class="space-y-2"
                                                            >
                                                                <h4
                                                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                                                                >
                                                                    <Layers
                                                                        class="h-3 w-3"
                                                                    />
                                                                    Header Changes
                                                                </h4>
                                                                <div
                                                                    class="space-y-2"
                                                                >
                                                                    <div
                                                                        v-for="change in vc(
                                                                            endpointData,
                                                                            v,
                                                                            index,
                                                                        ).headers"
                                                                        :key="
                                                                            change.name
                                                                        "
                                                                        class="flex items-start gap-2 text-sm"
                                                                    >
                                                                        <div
                                                                            v-if="
                                                                                change.type ===
                                                                                'added'
                                                                            "
                                                                            class="flex items-center gap-2 text-green-600 dark:text-green-400"
                                                                        >
                                                                            <Plus
                                                                                class="h-3 w-3"
                                                                            />
                                                                            <span
                                                                                class="font-medium underline decoration-dotted"
                                                                                >{{
                                                                                    change.name
                                                                                }}</span
                                                                            >
                                                                            <span
                                                                                class="opacity-70"
                                                                                >:
                                                                                {{
                                                                                    change.value
                                                                                }}</span
                                                                            >
                                                                        </div>
                                                                        <div
                                                                            v-else-if="
                                                                                change.type ===
                                                                                'removed'
                                                                            "
                                                                            class="flex items-center gap-2 text-red-600 dark:text-red-400 opacity-70"
                                                                        >
                                                                            <Minus
                                                                                class="h-3 w-3"
                                                                            />
                                                                            <span
                                                                                class="font-medium line-through"
                                                                                >{{
                                                                                    change.name
                                                                                }}</span
                                                                            >
                                                                        </div>
                                                                        <div
                                                                            v-else-if="
                                                                                change.type ===
                                                                                'changed'
                                                                            "
                                                                            class="flex flex-col gap-0.5"
                                                                        >
                                                                            <div
                                                                                class="flex items-center gap-2"
                                                                            >
                                                                                <span
                                                                                    class="font-medium text-muted-foreground"
                                                                                    >{{
                                                                                        change.name
                                                                                    }}</span
                                                                                >
                                                                                <span
                                                                                    class="text-[10px] bg-muted px-1 rounded"
                                                                                    >Changed</span
                                                                                >
                                                                            </div>
                                                                            <div
                                                                                class="pl-5 flex items-center gap-2 text-muted-foreground opacity-70"
                                                                            >
                                                                                <span
                                                                                    class="line-through"
                                                                                    >{{
                                                                                        change.oldValue
                                                                                    }}</span
                                                                                >
                                                                                <ArrowRight
                                                                                    class="h-3 w-3"
                                                                                />
                                                                                <span
                                                                                    class="text-foreground font-medium"
                                                                                    >{{
                                                                                        change.newValue
                                                                                    }}</span
                                                                                >
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div
                                                                v-if="
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).params
                                                                        .length > 0
                                                                "
                                                                class="space-y-2"
                                                            >
                                                                <h4
                                                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                                                                >
                                                                    <Plus
                                                                        class="h-3 w-3"
                                                                    />
                                                                    Parameter Changes
                                                                </h4>
                                                                <div
                                                                    class="space-y-2"
                                                                >
                                                                    <div
                                                                        v-for="change in vc(
                                                                            endpointData,
                                                                            v,
                                                                            index,
                                                                        ).params"
                                                                        :key="
                                                                            change.name
                                                                        "
                                                                        class="flex items-start gap-2 text-sm"
                                                                    >
                                                                        <div
                                                                            v-if="
                                                                                change.type ===
                                                                                'added'
                                                                            "
                                                                            class="flex items-center gap-2 text-green-600 dark:text-green-400"
                                                                        >
                                                                            <Plus
                                                                                class="h-3 w-3"
                                                                            />
                                                                            <span
                                                                                class="font-medium underline decoration-dotted"
                                                                                >{{
                                                                                    change.name
                                                                                }}</span
                                                                            >
                                                                            <span
                                                                                class="opacity-70"
                                                                                >:
                                                                                {{
                                                                                    change.value
                                                                                }}</span
                                                                            >
                                                                        </div>
                                                                        <div
                                                                            v-else-if="
                                                                                change.type ===
                                                                                'removed'
                                                                            "
                                                                            class="flex items-center gap-2 text-red-600 dark:text-red-400 opacity-70"
                                                                        >
                                                                            <Minus
                                                                                class="h-3 w-3"
                                                                            />
                                                                            <span
                                                                                class="font-medium line-through"
                                                                                >{{
                                                                                    change.name
                                                                                }}</span
                                                                            >
                                                                        </div>
                                                                        <div
                                                                            v-else-if="
                                                                                change.type ===
                                                                                'changed'
                                                                            "
                                                                            class="flex flex-col gap-0.5"
                                                                        >
                                                                            <div
                                                                                class="flex items-center gap-2"
                                                                            >
                                                                                <span
                                                                                    class="font-medium text-muted-foreground"
                                                                                    >{{
                                                                                        change.name
                                                                                    }}</span
                                                                                >
                                                                                <span
                                                                                    class="text-[10px] bg-muted px-1 rounded"
                                                                                    >Changed</span
                                                                                >
                                                                            </div>
                                                                            <div
                                                                                class="pl-5 flex items-center gap-2 text-muted-foreground opacity-70"
                                                                            >
                                                                                <span
                                                                                    class="line-through"
                                                                                    >{{
                                                                                        change.oldValue
                                                                                    }}</span
                                                                                >
                                                                                <ArrowRight
                                                                                    class="h-3 w-3"
                                                                                />
                                                                                <span
                                                                                    class="text-foreground font-medium"
                                                                                    >{{
                                                                                        change.newValue
                                                                                    }}</span
                                                                                >
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div
                                                                v-if="
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).body
                                                                "
                                                                class="space-y-2"
                                                            >
                                                                <h4
                                                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                                                                >
                                                                    <Layers
                                                                        class="h-3 w-3"
                                                                    />
                                                                    Body Changes
                                                                </h4>
                                                                <div
                                                                    class="text-sm space-y-2"
                                                                >
                                                                    <div
                                                                        class="p-2 rounded bg-red-500/5 border border-red-500/20 opacity-50 font-mono whitespace-pre-wrap max-h-40 overflow-auto"
                                                                    >
                                                                        {{
                                                                            vc(
                                                                                endpointData,
                                                                                v,
                                                                                index,
                                                                            ).body
                                                                                .old || "(empty)"
                                                                        }}
                                                                    </div>
                                                                    <div
                                                                        class="flex justify-center"
                                                                    >
                                                                        <ArrowRight
                                                                            class="h-4 w-4 text-muted-foreground rotate-90"
                                                                        />
                                                                    </div>
                                                                    <div
                                                                        class="p-2 rounded bg-green-500/5 border border-green-500/20 font-mono whitespace-pre-wrap max-h-40 overflow-auto"
                                                                    >
                                                                        {{
                                                                            vc(
                                                                                endpointData,
                                                                                v,
                                                                                index,
                                                                            ).body
                                                                                .new || "(empty)"
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div
                                                                v-if="
                                                                    vc(
                                                                        endpointData,
                                                                        v,
                                                                        index,
                                                                    ).preflight
                                                                "
                                                                class="space-y-2"
                                                            >
                                                                <h4
                                                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2"
                                                                >
                                                                    <RefreshCw
                                                                        class="h-3 w-3"
                                                                    />
                                                                    Preflight Script
                                                                    Changes
                                                                </h4>
                                                                <div
                                                                    class="space-y-1"
                                                                >
                                                                    <div
                                                                        v-for="(
                                                                            change,
                                                                            key,
                                                                        ) in vc(
                                                                            endpointData,
                                                                            v,
                                                                            index,
                                                                        ).preflight"
                                                                        :key="key"
                                                                        class="flex items-center gap-2 text-sm"
                                                                    >
                                                                        <span
                                                                            class="w-24 text-muted-foreground capitalize"
                                                                            >{{
                                                                                key
                                                                            }}:</span
                                                                        >
                                                                        <span
                                                                            class="line-through opacity-50"
                                                                            >{{
                                                                                change.old
                                                                            }}</span
                                                                        >
                                                                        <ArrowRight
                                                                            class="h-3 w-3 text-muted-foreground"
                                                                        />
                                                                        <span
                                                                            class="text-primary font-medium"
                                                                            >{{
                                                                                change.new
                                                                            }}</span
                                                                        >
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </template>
                                                    <div
                                                        v-else
                                                        class="text-sm text-muted-foreground italic bg-muted/30 p-2 rounded border border-dashed text-center"
                                                    >
                                                        Initial version. No
                                                        previous baseline for
                                                        comparison.
                                                    </div>
                                                </div>
                                            </AccordionContent>
                                        </AccordionItem>
                                    </Accordion>
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </AccordionContent>
                </AccordionItem>
            </Accordion>
        </template>
        <template v-else>
            <div
                class="flex flex-col items-center justify-center py-12 text-muted-foreground space-y-2"
            >
                <AlertCircle class="h-8 w-8 opacity-20" />
                <p>No version history available yet.</p>
                <p class="opacity-70 text-xs text-balance text-center">
                    New versions are created automatically when you save changes.
                </p>
            </div>
        </template>
    </div>
</template>

<style scoped>
:deep([data-slot="accordion-trigger"]) {
    padding-block: 0.5rem;
}
</style>