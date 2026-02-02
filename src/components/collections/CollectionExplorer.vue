<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { Search, Plus, X, Layers, PlusCircle } from "lucide-vue-next";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import ServiceTree from "@/components/ServiceTree.vue";
import { useCollectionsStore } from "@/stores/collections";
import { useDialogState } from "@/composables/useDialogState";

const emit = defineEmits<{
  (e: "select-endpoint", endpoint: any): void;
  (e: "select-service-settings", service: any): void;
  (e: "env-change", serviceId: string, env: string): void;
  (e: "endpoint-context", event: MouseEvent, endpoint: any): void;
}>();

const collectionsStore = useCollectionsStore();
const { isCollectionDialogOpen, isCollectionEndpointDialogOpen } =
  useDialogState();

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

// Handle global keydown for search shortcut
const handleGlobalKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    searchInput.value?.focus();
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeydown);
});

const filteredCollections = computed(() => {
  if (!searchQuery.value) return collectionsStore.collections;

  const query = searchQuery.value.toLowerCase();
  return collectionsStore.collections
    .map((collection) => {
      const collectionMatches = collection.name.toLowerCase().includes(query);
      const matchingEndpoints = collection.endpoints.filter(
        (endpoint) =>
          endpoint.name.toLowerCase().includes(query) ||
          endpoint.url.toLowerCase().includes(query) ||
          endpoint.method.toLowerCase().includes(query),
      );

      if (collectionMatches) return collection;
      if (matchingEndpoints.length > 0) {
        return {
          ...collection,
          endpoints: matchingEndpoints,
          isOpen: true,
        };
      }
      return null;
    })
    .filter((c): c is any => c !== null);
});
</script>

<template>
  <div class="flex flex-col h-full bg-muted/5 border-r">
    <div class="p-3 border-b flex items-center justify-between">
      <div class="relative flex-1 mr-2">
        <Search
          class="absolute left-2 top-2.5 h-3.5 w-3.5 text-muted-foreground"
        />
        <input
          ref="searchInput"
          v-model="searchQuery"
          type="text"
          placeholder="Search scratchpad..."
          class="w-full bg-background border rounded-md py-1.5 pl-7 pr-8 focus:outline-none focus:ring-1 focus:ring-primary"
        />
        <button
          v-if="searchQuery"
          @click="searchQuery = ''"
          class="absolute right-2 top-2 p-0.5 hover:bg-muted rounded-sm transition-colors"
        >
          <X class="h-3 w-3 text-muted-foreground" />
        </button>
      </div>

      <div class="flex items-center gap-1">
        <Popover>
          <PopoverTrigger as-child>
            <button
              class="p-1.5 hover:bg-muted rounded-md transition-colors text-muted-foreground"
            >
              <Plus class="h-4 w-4" />
            </button>
          </PopoverTrigger>
          <PopoverContent class="w-48 p-1" align="end">
            <div class="flex flex-col">
              <button
                @click="isCollectionDialogOpen = true"
                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
              >
                <Layers class="h-3.5 w-3.5 text-primary" />
                <span>New Collection</span>
              </button>
              <button
                @click="isCollectionEndpointDialogOpen = true"
                class="flex items-center gap-2 px-2 py-2 hover:bg-muted rounded-sm text-left transition-colors"
              >
                <PlusCircle class="h-3.5 w-3.5 text-green-500" />
                <span>New Endpoint</span>
              </button>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
    <div class="flex-1 overflow-auto p-2">
      <div
        v-if="collectionsStore.collections.length === 0"
        class="h-full flex flex-col items-center justify-center p-4 text-center space-y-2 opacity-50"
      >
        <Layers class="h-8 w-8 mb-2" />
        <p class="font-medium leading-tight">API Scratchpad</p>
        <p class="">
          Create a collection to start exploring APIs without git or
          directories.
        </p>
      </div>
      <ServiceTree
        v-else
        :services="filteredCollections"
        @select-endpoint="emit('select-endpoint', $event)"
        @select-service-settings="emit('select-service-settings', $event)"
        @env-change="(id, env) => emit('env-change', id, env)"
        @endpoint-context="(e, ep) => emit('endpoint-context', e, ep)"
      />
    </div>
  </div>
</template>
