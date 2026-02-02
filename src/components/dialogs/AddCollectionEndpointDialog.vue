<script setup lang="ts">
import { ref } from "vue";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

defineProps<{
  open: boolean;
  collections: any[];
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "endpoint-created", endpoint: any, collectionId: string): void;
}>();

const newEndpointName = ref("");
const newEndpointMethod = ref("GET");
const newEndpointPath = ref("");
const selectedCollectionId = ref("");

const handleAddEndpoint = () => {
  if (!newEndpointName.value.trim()) return;

  const newEndpoint: any = {
    id: `e-${Date.now()}`,
    serviceId: selectedCollectionId.value,
    name: newEndpointName.value,
    method: newEndpointMethod.value,
    url: newEndpointPath.value || "/",
    authenticated: false,
    authType: "none",
    metadata: {
      version: "1.0",
      lastUpdated: Date.now(),
    },
    params: [],
    headers: [],
    body: "",
    preflight: {
      enabled: false,
      method: "GET",
      url: "",
      body: "",
      headers: [],
      cacheToken: true,
      cacheDuration: "derived",
      cacheDurationKey: "expires_in",
      cacheDurationUnit: "seconds",
      tokenKey: "access_token",
      tokenHeader: "Authorization",
    },
  };

  emit("endpoint-created", newEndpoint, selectedCollectionId.value);
  emit("update:open", false);

  // Reset form
  newEndpointName.value = "";
  newEndpointMethod.value = "GET";
  newEndpointPath.value = "";
  selectedCollectionId.value = "";
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[425px]">
      <DialogHeader>
        <DialogTitle>Add New Endpoint</DialogTitle>
        <DialogDescription>
          Configure the details for your new endpoint. Choose a collection or
          leave empty to create a new one.
        </DialogDescription>
      </DialogHeader>
      <div class="grid gap-4 py-4">
        <div class="grid grid-cols-4 items-center gap-4">
          <Label class="text-right">Collection</Label>
          <Select v-model="selectedCollectionId">
            <SelectTrigger class="col-span-3 h-8">
              <SelectValue placeholder="Select a collection (optional)" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>Collections</SelectLabel>
                <SelectItem
                  v-for="collection in collections"
                  :key="collection.id"
                  :value="collection.id"
                  class=""
                >
                  {{ collection.name }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div class="grid grid-cols-4 items-center gap-4">
          <Label for="endpoint-name-coll" class="text-right">Name</Label>
          <Input
            id="endpoint-name-coll"
            v-model="newEndpointName"
            placeholder="E.g. Get User Profile"
            class="col-span-3 h-8"
            @keyup.enter="handleAddEndpoint"
          />
        </div>
        <div class="grid grid-cols-4 items-center gap-4">
          <Label class="text-right">Method</Label>
          <Select v-model="newEndpointMethod">
            <SelectTrigger class="col-span-3 h-8">
              <SelectValue placeholder="GET" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="GET" class="font-bold text-green-600"
                >GET</SelectItem
              >
              <SelectItem value="POST" class="font-bold text-orange-600"
                >POST</SelectItem
              >
              <SelectItem value="PUT" class="font-bold text-blue-600"
                >PUT</SelectItem
              >
              <SelectItem value="PATCH" class="font-bold text-yellow-600"
                >PATCH</SelectItem
              >
              <SelectItem value="DELETE" class="font-bold text-red-600"
                >DELETE</SelectItem
              >
            </SelectContent>
          </Select>
        </div>
        <div class="grid grid-cols-4 items-center gap-4">
          <Label for="endpoint-path-coll" class="text-right">URL/Path</Label>
          <Input
            id="endpoint-path-coll"
            v-model="newEndpointPath"
            placeholder="https://api.example.com/endpoint"
            class="col-span-3 h-8"
            @keyup.enter="handleAddEndpoint"
          />
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" size="sm" @click="emit('update:open', false)"
          >Cancel</Button
        >
        <Button size="sm" @click="handleAddEndpoint">Create Endpoint</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
