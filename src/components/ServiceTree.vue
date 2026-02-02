<script setup lang="ts">
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Endpoint, Service } from "@/types";
import {
  Check,
  ChevronDown,
  ChevronDown as ChevronDownIcon,
  ChevronRight,
  Layers,
  Settings
} from "lucide-vue-next";
import { ref, watch } from "vue";

const props = defineProps<{
  services: Service[];
  showEnvironments?: boolean;
}>();

const emits = defineEmits<{
  (e: "select-endpoint", endpoint: Endpoint): void;
  (e: "select-service-settings", service: Service): void;
  (e: "env-change", serviceId: string, env: string): void;
  (e: "endpoint-context", event: MouseEvent, endpoint: Endpoint): void;
  (e: "import-curl", service: Service): void;
}>();

const localServices = ref(props.services.map((s) => ({ ...s, isOpen: true })));

watch(
  () => props.services,
  (newServices) => {
    localServices.value = newServices.map((ns) => {
      const existing = localServices.value.find((ls) => ls.id === ns.id);
      const forceOpen = (ns as any).isOpen;
      return {
        ...ns,
        isOpen:
          forceOpen !== undefined
            ? forceOpen
            : existing
              ? existing.isOpen
              : true,
      };
    });
  },
  { deep: true },
);

const toggleService = (index: number) => {
  localServices.value[index].isOpen = !localServices.value[index].isOpen;
};

const getEnvironments = (service: Service) => {
  if (!service.environments || service.environments.length === 0)
    return ["DEFAULT"];
  return service.environments.map((e) => e.name);
};

const handleEnvChange = (serviceId: string, value: any) => {
  emits("env-change", serviceId, String(value));
};

const getMethodColor = (method: string) => {
  switch (method?.toUpperCase()) {
    case "GET":
      return "text-green-600";
    case "POST":
      return "text-orange-600";
    case "PUT":
      return "text-blue-600";
    case "DELETE":
      return "text-red-600";
    default:
      return "text-muted-foreground";
  }
};

const isEnvUnsafe = (service: Service, envName: string) => {
  const env = service.environments?.find((e) => e.name === envName);
  return env?.isUnsafe || false;
};
</script>

<template>
  <div class="space-y-1">
    <div v-for="(service, sIdx) in localServices" :key="service.id" class="select-none">
      <!-- Service Header -->
      <div class="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-muted/50 group transition-colors">
        <div @click="toggleService(sIdx)" class="flex items-center gap-1.5 flex-1 cursor-pointer overflow-hidden">
          <component :is="service.isOpen ? ChevronDown : ChevronRight"
            class="h-3.5 w-3.5 text-muted-foreground shrink-0" />
          <Layers class="h-3.5 w-3.5 text-primary/70 shrink-0" />
          <span class="font-semibold truncate">{{ service.name }}</span>
        </div>

        <Popover>
          <PopoverTrigger as-child>
            <button v-if="service.environments && service.environments.length > 0" :class="[
              'h-6 flex items-center gap-1.5 px-2 font-bold rounded border transition-all outline-none',
              isEnvUnsafe(
                service,
                service.selectedEnvironment || service.environments[0].name,
              )
                ? 'text-destructive bg-destructive/10 border-destructive/20 hover:bg-destructive/20 hover:border-destructive/30'
                : 'text-muted-foreground hover:text-primary bg-muted/30 hover:bg-primary/5 border-transparent hover:border-primary/20',
            ]">
              <span class="truncate max-w-[50px]">{{
                service.selectedEnvironment || service.environments[0].name
              }}</span>
              <ChevronDownIcon class="h-3 w-3 opacity-50" />
            </button>
          </PopoverTrigger>
          <PopoverContent class="w-28 p-1" align="end">
            <div class="flex flex-col gap-0.5">
              <button v-for="env in getEnvironments(service)" :key="env" @click="handleEnvChange(service.id, env)"
                :class="[
                  'flex items-center justify-between px-2 py-1.5 rounded-sm transition-colors',
                  (service.selectedEnvironment ||
                    service.environments[0].name) === env
                    ? isEnvUnsafe(service, env)
                      ? 'bg-destructive/10 text-destructive font-bold'
                      : 'bg-primary/10 text-primary font-bold'
                    : isEnvUnsafe(service, env)
                      ? 'text-destructive/70 hover:bg-destructive/5 hover:text-destructive'
                      : 'hover:bg-muted text-muted-foreground hover:text-foreground',
                ]">
                <span>{{ env }}</span>
                <Check v-if="
                  (service.selectedEnvironment ||
                    service.environments[0].name) === env
                " class="h-3 w-3" />
              </button>
            </div>
          </PopoverContent>
        </Popover>

        <button @click="emits('select-service-settings', service)"
          class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors opacity-0 group-hover:opacity-100"
          title="Service Settings">
          <Settings class="h-3.5 w-3.5" />
        </button>
      </div>

      <div v-if="service.isOpen" class="ml-4 mt-0.5 space-y-0.5 border-l border-border/50 pl-2">
        <div v-for="endpoint in service.endpoints" :key="endpoint.id" @click="emits('select-endpoint', endpoint)"
          @contextmenu.prevent="emits('endpoint-context', $event, endpoint)"
          class="flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-muted cursor-pointer group transition-colors">
          <div :class="[
            'text-[10px] font-bold w-14 shrink-0 uppercase text-left',
            getMethodColor(endpoint.method),
          ]">
            {{ endpoint.method }}
          </div>
          <span class="text-foreground/80 group-hover:text-foreground truncate">{{ endpoint.name }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
