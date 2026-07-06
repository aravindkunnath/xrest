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
import InterpolatedInput from "@/components/InterpolatedInput.vue";

defineProps<{
  open: boolean;
  services: any[];
  allActiveVariables: Record<string, Record<string, string>>;
  activeEnvironments: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "endpoint-created", endpoint: any, serviceId: string): void;
}>();

const newEndpointName = ref("");
const newEndpointMethod = ref("GET");
const newEndpointPath = ref("");
const selectedServiceIdForNewEndpoint = ref("");

const handleAddEndpoint = () => {
  if (!newEndpointName.value.trim() || !selectedServiceIdForNewEndpoint.value)
    return;

  const newEndpoint: any = {
    id: `e-${Date.now()}`,
    serviceId: selectedServiceIdForNewEndpoint.value,
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

  emit("endpoint-created", newEndpoint, selectedServiceIdForNewEndpoint.value);

  // Reset form
  newEndpointName.value = "";
  newEndpointMethod.value = "GET";
  newEndpointPath.value = "";
  selectedServiceIdForNewEndpoint.value = "";
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[425px]">
      <DialogHeader>
        <DialogTitle>Add New Endpoint</DialogTitle>
        <DialogDescription>
          Choose a service and configure the details for your new endpoint.
        </DialogDescription>
      </DialogHeader>
      <div class="grid gap-4 py-4">
        <div class="grid grid-cols-4 items-center gap-4">
          <Label class="text-right">Service</Label>
          <Select v-model="selectedServiceIdForNewEndpoint">
            <SelectTrigger class="col-span-3 h-8">
              <SelectValue placeholder="Select a service" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>Services</SelectLabel>
                <SelectItem
                  v-for="service in services"
                  :key="service.id"
                  :value="service.id"
                  class=""
                >
                  {{ service.name }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div class="grid grid-cols-4 items-center gap-4">
          <Label for="endpoint-name" class="text-right">Name</Label>
          <Input
            id="endpoint-name"
            v-model="newEndpointName"
            placeholder="E.g. Get User Profile"
            class="col-span-3 h-8"
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
          <Label for="endpoint-path" class="text-right">Path</Label>
          <InterpolatedInput
            id="endpoint-path"
            v-model="newEndpointPath"
            :variables="
              selectedServiceIdForNewEndpoint
                ? allActiveVariables[selectedServiceIdForNewEndpoint]
                : {}
            "
            :environment-name="
              selectedServiceIdForNewEndpoint
                ? activeEnvironments[selectedServiceIdForNewEndpoint]
                : ''
            "
            placeholder="/api/v1/resource"
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
