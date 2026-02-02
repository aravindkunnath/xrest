<script setup lang="ts">
import { onMounted, ref, computed, watch } from "vue";
import { useHistoryStore } from "@/stores/history";
import { useServicesStore } from "@/stores/services";
import { useCollectionsStore } from "@/stores/collections";
import { Search, Filter, X, ChevronDown } from "lucide-vue-next";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import JsonHighlighter from "@/components/JsonHighlighter.vue";

const historyStore = useHistoryStore();
const servicesStore = useServicesStore();
const collectionsStore = useCollectionsStore();

const searchQuery = ref("");
const selectedServiceId = ref("all");
const selectedEndpointId = ref("all");
const selectedMethod = ref("all");

const methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

onMounted(async () => {
  await historyStore.fetchHistory();
  if (servicesStore.services.length === 0) {
    await servicesStore.loadServices();
  }
  if (collectionsStore.collections.length === 0) {
    await collectionsStore.loadCollections();
  }
});

// Reset endpoint filter if service changes
watch(selectedServiceId, () => {
  selectedEndpointId.value = "all";
});

const services = computed(() => servicesStore.services);
const collections = computed(() => collectionsStore.collections);

const endpointsForSelectedService = computed(() => {
  if (selectedServiceId.value === "all") return [];
  const service = services.value.find((s) => s.id === selectedServiceId.value);
  if (service) return service.endpoints;

  const collection = collections.value.find(
    (c) => c.id === selectedServiceId.value,
  );
  return collection ? collection.endpoints : [];
});

const filteredHistory = computed(() => {
  return historyStore.history.filter((item) => {
    // Service filter
    if (
      selectedServiceId.value !== "all" &&
      item.serviceId !== selectedServiceId.value
    )
      return false;

    // Endpoint filter
    if (
      selectedEndpointId.value !== "all" &&
      item.endpointId !== selectedEndpointId.value
    )
      return false;

    // Method filter
    if (
      selectedMethod.value !== "all" &&
      item.method.toUpperCase() !== selectedMethod.value
    )
      return false;

    // Text search (URL, Headers, Bodies)
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      const inUrl = item.url.toLowerCase().includes(q);
      const inRequestBody = item.requestBody?.toLowerCase().includes(q);
      const inResponseBody = item.responseBody?.toLowerCase().includes(q);
      const inHeaders = [
        ...(item.requestHeaders || []),
        ...(item.responseHeaders || []),
      ].some(
        (h) =>
          h.name.toLowerCase().includes(q) || h.value.toLowerCase().includes(q),
      );

      if (!inUrl && !inRequestBody && !inResponseBody && !inHeaders)
        return false;
    }

    return true;
  });
});

function getServiceName(id: string | null) {
  if (!id) return "Unknown Source";
  const service = services.value.find((s) => s.id === id);
  if (service) return service.name;

  const collection = collections.value.find((c) => c.id === id);
  return collection ? `(Coll) ${collection.name}` : "Unknown Source";
}

function getEndpointName(serviceId: string | null, endpointId: string | null) {
  if (!serviceId || !endpointId) return null;
  const service = services.value.find((s) => s.id === serviceId);
  if (service) {
    const endpoint = service.endpoints.find((e) => e.id === endpointId);
    return endpoint ? endpoint.name : null;
  }

  const collection = collections.value.find((c) => c.id === serviceId);
  if (collection) {
    const endpoint = collection.endpoints.find((e) => e.id === endpointId);
    return endpoint ? endpoint.name : null;
  }

  return null;
}

function getMethodColor(method: string) {
  const m = method.toUpperCase();
  if (m === "GET") return "text-green-500";
  if (m === "POST") return "text-blue-500";
  if (m === "PUT" || m === "PATCH") return "text-yellow-500";
  if (m === "DELETE") return "text-red-500";
  return "text-muted-foreground";
}

function getStatusColor(status: number) {
  if (status >= 200 && status < 300) return "text-green-600 bg-green-500/10";
  if (status >= 400) return "text-red-600 bg-red-500/10";
  if (status >= 300) return "text-yellow-600 bg-yellow-500/10";
  return "text-muted-foreground bg-muted";
}

function formatRelativeTime(dateString: string) {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    const minutes = Math.floor(diff / 60000);
    if (minutes < 1) return "just now";
    if (minutes < 60) return `${minutes}m ago`;

    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;

    return date.toLocaleDateString();
  } catch (e) {
    return "unknown";
  }
}

function filterHeaders(headers: { name: string; value: string }[]) {
  return (headers || []).filter(
    (h) => h.name.toLowerCase() !== "authorization",
  );
}

function formatSize(bytes: number) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / 1048576).toFixed(1) + " MB";
}

function clearFilters() {
  searchQuery.value = "";
  selectedServiceId.value = "all";
  selectedEndpointId.value = "all";
  selectedMethod.value = "all";
}
</script>

<template>
  <div class="flex flex-col h-full w-full overflow-hidden">
    <div class="px-6 py-6 shrink-0 border-b bg-card/30 space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold tracking-tight">History</h1>
          <p class="text-muted-foreground mt-1">
            Review/audit your previous microservice interactions.
          </p>
        </div>
        <div class="flex items-center gap-3">
          <button
            @click="historyStore.fetchHistory()"
            class="inline-flex items-center justify-center rounded-md px-3 py-1.5 font-medium border bg-background hover:bg-accent transition-colors disabled:opacity-50"
            :disabled="historyStore.isLoading"
          >
            {{ historyStore.isLoading ? "Loading..." : "Refresh" }}
          </button>
          <button
            @click="historyStore.clearHistory()"
            class="inline-flex items-center justify-center rounded-md px-3 py-1.5 font-medium text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-50"
            :disabled="historyStore.isLoading"
          >
            Clear All
          </button>
        </div>
      </div>

      <!-- Filter Bar -->
      <div class="flex flex-wrap items-center gap-3 pt-2">
        <!-- Search -->
        <div class="relative flex-1 min-w-[300px]">
          <Search
            class="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60"
          />
          <input
            v-model="searchQuery"
            placeholder="Search URL, headers, or body..."
            class="w-full bg-background border rounded-md pl-9 pr-8 py-2 focus:outline-none focus:ring-1 focus:ring-ring transition-all"
          />
          <button
            v-if="searchQuery"
            @click="searchQuery = ''"
            class="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-accent rounded-md text-muted-foreground transition-colors"
          >
            <X class="h-3 w-3" />
          </button>
        </div>

        <!-- Source Filter -->
        <div class="relative min-w-[160px]">
          <select
            v-model="selectedServiceId"
            class="w-full appearance-none bg-background border rounded-md px-3 py-2 focus:outline-none focus:ring-1 focus:ring-ring cursor-pointer pr-10"
          >
            <option value="all">All Sources</option>
            <optgroup label="Services">
              <option
                v-for="service in services"
                :key="service.id"
                :value="service.id"
              >
                {{ service.name }}
              </option>
            </optgroup>
            <optgroup label="Collections">
              <option
                v-for="collection in collections"
                :key="collection.id"
                :value="collection.id"
              >
                {{ collection.name }}
              </option>
            </optgroup>
          </select>
          <ChevronDown
            class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60 pointer-events-none"
          />
        </div>

        <!-- Endpoint Filter -->
        <div class="relative min-w-[160px]">
          <select
            v-model="selectedEndpointId"
            :disabled="
              selectedServiceId === 'all' ||
              endpointsForSelectedService.length === 0
            "
            class="w-full appearance-none bg-background border rounded-md px-3 py-2 focus:outline-none focus:ring-1 focus:ring-ring cursor-pointer pr-10 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <option value="all">All Endpoints</option>
            <option
              v-for="endpoint in endpointsForSelectedService"
              :key="endpoint.id"
              :value="endpoint.id"
            >
              {{ endpoint.name }}
            </option>
          </select>
          <ChevronDown
            class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60 pointer-events-none"
          />
        </div>

        <!-- Method Filter -->
        <div class="relative min-w-[120px]">
          <select
            v-model="selectedMethod"
            class="w-full appearance-none bg-background border rounded-md px-3 py-2 focus:outline-none focus:ring-1 focus:ring-ring cursor-pointer pr-10"
          >
            <option value="all">All Methods</option>
            <option v-for="m in methods" :key="m" :value="m">{{ m }}</option>
          </select>
          <ChevronDown
            class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60 pointer-events-none"
          />
        </div>

        <button
          @click="clearFilters"
          v-if="
            searchQuery ||
            selectedServiceId !== 'all' ||
            selectedMethod !== 'all'
          "
          class="p-2 text-muted-foreground hover:text-foreground transition-colors"
          title="Clear Filters"
        >
          <Filter class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto w-full px-6 py-4">
      <div
        v-if="filteredHistory.length === 0"
        class="text-center py-24 border-2 border-dashed rounded-xl bg-muted/20"
      >
        <p class="text-muted-foreground">
          {{
            historyStore.history.length === 0
              ? "No history yet. Start making requests!"
              : "No results matching your filters."
          }}
        </p>
        <button
          v-if="historyStore.history.length > 0"
          @click="clearFilters"
          class="mt-4 font-medium text-primary hover:underline"
        >
          Clear all filters
        </button>
      </div>

      <div v-else class="w-full space-y-2 pb-6">
        <Accordion type="single" collapsible class="w-full space-y-2">
          <AccordionItem
            v-for="item in filteredHistory as any"
            :key="item.id"
            :value="item.id"
            class="border rounded-lg bg-card/50 overflow-hidden"
          >
            <AccordionTrigger
              class="px-4 py-3 hover:no-underline hover:bg-accent/30 transition-colors"
            >
              <div class="flex items-center gap-4 w-full text-left">
                <span
                  class="font-bold min-w-[50px] px-2 py-1 rounded bg-muted/50 text-center shrink-0"
                  :class="getMethodColor(item.method)"
                >
                  {{ item.method }}
                </span>
                <div class="flex flex-col min-w-0 flex-1">
                  <span
                    class="font-medium truncate opacity-90 tracking-tight"
                    :title="item.url"
                  >
                    {{ item.url }}
                  </span>
                  <div class="flex items-center gap-2 mt-0.5">
                    <span class="text-muted-foreground/60 font-medium">{{
                      getServiceName(item.serviceId)
                    }}</span>
                    <span
                      v-if="getEndpointName(item.serviceId, item.endpointId)"
                      class="text-muted-foreground/40 shrink-0"
                      >/</span
                    >
                    <span
                      v-if="getEndpointName(item.serviceId, item.endpointId)"
                      class="text-muted-foreground/60 font-medium"
                      >{{
                        getEndpointName(item.serviceId, item.endpointId)
                      }}</span
                    >
                  </div>
                </div>
                <div class="flex items-center gap-4 pr-4">
                  <span
                    class="px-2 py-0.5 rounded font-bold tracking-tight min-w-[32px] text-center"
                    :class="getStatusColor(item.responseStatus)"
                  >
                    {{ item.responseStatus }}
                  </span>
                  <span
                    class="text-muted-foreground tabular-nums min-w-[80px] text-right"
                  >
                    {{ formatRelativeTime(item.createdAt) }}
                  </span>
                </div>
              </div>
            </AccordionTrigger>
            <AccordionContent class="px-0 border-t bg-muted/5">
              <div
                class="grid grid-cols-1 xl:grid-cols-2 divide-y xl:divide-y-0 xl:divide-x border-b last:border-0"
              >
                <!-- Request Side -->
                <div class="p-6 space-y-6">
                  <div class="flex items-center justify-between border-b pb-2">
                    <h3
                      class="font-bold uppercase tracking-[0.2em] text-muted-foreground"
                    >
                      Request Details
                    </h3>
                    <div class="flex items-center gap-3">
                      <span
                        class="font-bold"
                        :class="getMethodColor(item.method)"
                        >{{ item.method }}</span
                      >
                    </div>
                  </div>

                  <div
                    v-if="filterHeaders(item.requestHeaders).length > 0"
                    class="space-y-2"
                  >
                    <p
                      class="font-bold text-muted-foreground/60 uppercase tracking-widest"
                    >
                      Headers
                    </p>
                    <div
                      class="bg-background/50 border rounded-md p-3 space-y-1.5 shadow-sm"
                    >
                      <div
                        v-for="header in filterHeaders(item.requestHeaders)"
                        :key="header.name"
                        class="flex items-baseline gap-3"
                      >
                        <span
                          class="text-muted-foreground/80 min-w-[120px] select-all"
                          >{{ header.name }}</span
                        >
                        <span class="text-foreground shrink-0 opacity-20"
                          >:</span
                        >
                        <span class="text-foreground/90 break-all select-all">{{
                          header.value
                        }}</span>
                      </div>
                    </div>
                  </div>

                  <div v-if="item.requestBody" class="space-y-2">
                    <p
                      class="font-bold text-muted-foreground/60 uppercase tracking-widest"
                    >
                      Payload
                    </p>
                    <JsonHighlighter
                      :code="item.requestBody"
                      class="max-h-[300px]"
                    />
                  </div>
                </div>

                <!-- Response Side -->
                <div class="p-6 space-y-6 bg-accent/[0.02]">
                  <div class="flex items-center justify-between border-b pb-2">
                    <h3
                      class="font-bold uppercase tracking-[0.2em] text-muted-foreground"
                    >
                      Response Summary
                    </h3>
                    <div class="flex items-center gap-3">
                      <span
                        class="text-muted-foreground bg-muted px-1.5 py-0.5 rounded"
                        >{{ item.timeElapsed }}ms</span
                      >
                      <span
                        class="text-muted-foreground bg-muted px-1.5 py-0.5 rounded"
                        >{{ formatSize(item.size) }}</span
                      >
                    </div>
                  </div>

                  <div v-if="item.responseHeaders.length > 0" class="space-y-2">
                    <p
                      class="font-bold text-muted-foreground/60 uppercase tracking-widest"
                    >
                      Headers
                    </p>
                    <div
                      class="bg-background/50 border rounded-md p-3 space-y-1.5 shadow-sm overflow-hidden"
                    >
                      <div
                        v-for="header in item.responseHeaders"
                        :key="header.name"
                        class="flex items-baseline gap-3"
                      >
                        <span
                          class="text-muted-foreground/80 min-w-[120px] select-all"
                          >{{ header.name }}</span
                        >
                        <span class="text-foreground shrink-0 opacity-20"
                          >:</span
                        >
                        <span class="text-foreground/90 break-all select-all">{{
                          header.value
                        }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <p
                        class="font-bold text-muted-foreground/60 uppercase tracking-widest"
                      >
                        Response Body
                      </p>
                      <span
                        class="px-2 py-0.5 rounded font-bold uppercase tracking-wider shadow-sm"
                        :class="getStatusColor(item.responseStatus)"
                      >
                        {{ item.responseStatusText }}
                      </span>
                    </div>
                    <JsonHighlighter
                      :code="item.responseBody"
                      class="max-h-[500px]"
                    />
                  </div>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>
    </div>
  </div>
</template>

<style scoped>
@reference "../style.css";

:deep(.accordion-trigger-icon) {
  @apply text-muted-foreground/50 transition-transform duration-200;
}

[data-state="open"] :deep(.accordion-trigger-icon) {
  @apply rotate-180;
}
</style>
