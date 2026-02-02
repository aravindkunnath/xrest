<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Checkbox } from "@/components/ui/checkbox";
import {
  ArrowRight,
  Layers,
  Plus,
  AlertCircle,
  Info,
  ChevronRight,
} from "lucide-vue-next";
import { Service, Endpoint } from "@/types";
import { useServicesStore } from "@/stores/services";

const props = defineProps<{
  open: boolean;
  endpoint: Endpoint | null;
  sourceCollection: Service | null;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (
    e: "added",
    targetServiceId: string,
    updatedEndpoint: Endpoint,
    newVariables: { name: string; value: string }[],
  ): void;
}>();

const servicesStore = useServicesStore();
const selectedServiceId = ref("");

// Detected variables from endpoint
const detectedVariables = computed(() => {
  if (!props.endpoint) return [];
  const vars = new Set<string>();
  const regex = /\{\{([^}]+)\}\}/g;

  const searchIn = [
    props.endpoint.url,
    props.endpoint.body,
    ...(props.endpoint.params || []).map(
      (p: any) => (p.name || "") + (p.value || ""),
    ),
    ...(props.endpoint.headers || []).map(
      (h: any) => (h.name || "") + (h.value || ""),
    ),
  ];

  searchIn.forEach((text) => {
    if (!text) return;
    let match;
    while ((match = regex.exec(text)) !== null) {
      vars.add(match[1].trim());
    }
  });

  return Array.from(vars);
});

// Variables to be created in target service
const variablesToCreate = ref<
  { name: string; value: string; selected: boolean }[]
>([]);

watch(
  [() => props.endpoint, () => props.sourceCollection, () => props.open],
  () => {
    if (!props.endpoint || !props.sourceCollection || !props.open) return;

    const sourceEnv = props.sourceCollection.environments.find(
      (e) =>
        e.name === (props.sourceCollection?.selectedEnvironment || "GLOBAL"),
    );

    variablesToCreate.value = detectedVariables.value.map((v) => {
      const sourceVar = sourceEnv?.variables.find((sv) => sv.name === v);
      return {
        name: v,
        value: sourceVar ? sourceVar.value : "",
        selected: true,
      };
    });
  },
  { immediate: true },
);

const targetService = computed(() =>
  servicesStore.services.find((s) => s.id === selectedServiceId.value),
);

// Existing variable names in target service to show warnings/conflicts
const targetExistingVars = computed(() => {
  if (!targetService.value) return new Set<string>();
  const names = new Set<string>();
  targetService.value.environments.forEach((env) => {
    env.variables.forEach((v) => names.add(v.name));
  });
  return names;
});

const handleAdd = () => {
  if (!props.endpoint || !selectedServiceId.value || !targetService.value)
    return;

  const varsToAdd = variablesToCreate.value
    .filter((v) => v.selected && v.name.trim())
    .map((v) => ({ name: v.name, value: v.value }));

  // Create a copy of the endpoint and set its serviceId correctly
  const updatedEndpoint = {
    ...props.endpoint,
    serviceId: selectedServiceId.value,
  };

  emit("added", selectedServiceId.value, updatedEndpoint, varsToAdd);
  emit("update:open", false);

  // Reset
  selectedServiceId.value = "";
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[700px] gap-0 p-0 overflow-hidden">
      <DialogHeader class="p-6 pb-4 border-b">
        <DialogTitle class="flex items-center gap-2">
          <ArrowRight class="h-5 w-5 text-primary" />
          Add Endpoint to Service
        </DialogTitle>
        <DialogDescription>
          Migrate "{{ endpoint?.name }}" from collections to a permanent
          service.
        </DialogDescription>
      </DialogHeader>

      <div class="flex flex-col max-h-[70vh]">
        <div class="px-6 py-4 space-y-6 overflow-y-auto">
          <!-- Step 1: Target Service -->
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <span
                class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 font-bold text-primary"
                >1</span
              >
              <Label
                class="font-bold uppercase tracking-wider text-muted-foreground"
                >Select Target Service</Label
              >
            </div>
            <Select v-model="selectedServiceId">
              <SelectTrigger class="w-full h-10">
                <SelectValue placeholder="Choose a service..." />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectLabel>Available Services</SelectLabel>
                  <SelectItem
                    v-for="service in servicesStore.services"
                    :key="service.id"
                    :value="service.id"
                  >
                    <div class="flex items-center gap-2">
                      <Layers class="h-3.5 w-3.5 opacity-70" />
                      <span>{{ service.name }}</span>
                    </div>
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
            <p
              v-if="!selectedServiceId"
              class="text-muted-foreground flex items-center gap-1.5"
            >
              <Info class="h-3 w-3" /> Select a service to see environment
              compatibility.
            </p>
          </div>

          <Separator />

          <!-- Step 2: Variables -->
          <div
            class="space-y-4"
            :class="{ 'opacity-50 pointer-events-none': !selectedServiceId }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span
                  class="flex h-5 w-5 items-center justify-center rounded-full bg-primary/10 font-bold text-primary"
                  >2</span
                >
                <Label
                  class="font-bold uppercase tracking-wider text-muted-foreground"
                  >Merge Environment Variables</Label
                >
              </div>
            </div>

            <div v-if="variablesToCreate.length > 0">
              <div class="rounded-md border bg-muted/30">
                <Table>
                  <TableHeader>
                    <TableRow class="hover:bg-transparent">
                      <TableHead class="w-12"></TableHead>
                      <TableHead class="uppercase font-bold"
                        >Variable Name</TableHead
                      >
                      <TableHead class="uppercase font-bold"
                        >Current Value</TableHead
                      >
                      <TableHead class="w-32 uppercase font-bold"
                        >Status</TableHead
                      >
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="v in variablesToCreate"
                      :key="v.name"
                      class="group"
                    >
                      <TableCell>
                        <Checkbox v-model:checked="v.selected" />
                      </TableCell>
                      <TableCell class="py-2">
                        <Input v-model="v.name" class="h-7 bg-background" />
                      </TableCell>
                      <TableCell class="py-2">
                        <Input v-model="v.value" class="h-7 bg-background" />
                      </TableCell>
                      <TableCell>
                        <div
                          v-if="targetExistingVars.has(v.name)"
                          class="flex items-center gap-1 text-orange-500"
                        >
                          <AlertCircle class="h-3 w-3" />
                          <span class="font-bold uppercase truncate"
                            >Conflict</span
                          >
                        </div>
                        <div
                          v-else
                          class="flex items-center gap-1 text-green-600"
                        >
                          <Plus class="h-3 w-3" />
                          <span class="font-bold uppercase tracking-tight"
                            >New</span
                          >
                        </div>
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </div>
              <p
                class="mt-2 text-muted-foreground flex items-center gap-1.5 px-1"
              >
                <Info class="h-3 w-3 shrink-0" /> Selected variables will be
                added to all environments of "{{
                  targetService?.name || "the target service"
                }}".
              </p>
            </div>
            <div
              v-else
              class="py-8 text-center border-2 border-dashed rounded-md bg-muted/20"
            >
              <p class="text-muted-foreground">
                No variables detected in this endpoint.
              </p>
            </div>
          </div>
        </div>

        <!-- Footer Summary -->
        <div class="p-6 bg-muted/50 border-t mt-auto">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <div
                class="px-2 py-1 bg-background rounded border font-bold text-primary"
              >
                API Scratchpad
              </div>
              <ChevronRight class="h-4 w-4 text-muted-foreground" />
              <div
                class="px-2 py-1 bg-primary/10 rounded border border-primary/20 font-bold text-primary"
              >
                {{ targetService?.name || "Target Service" }}
              </div>
            </div>
            <div class="text-muted-foreground font-medium">
              The endpoint will be moved and removed from the scratchpad.
            </div>
          </div>

          <div class="flex justify-end gap-3">
            <Button variant="outline" @click="emit('update:open', false)">
              Cancel
            </Button>
            <Button
              :disabled="!selectedServiceId"
              @click="handleAdd"
              class="gap-2"
            >
              Migrate Endpoint
              <ArrowRight class="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
