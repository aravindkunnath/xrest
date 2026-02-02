<script setup lang="ts">
import { ref } from "vue";
import { toast } from "vue-sonner";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { ChevronLeft, ChevronRight, Folder, Plus } from "lucide-vue-next";
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
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ServiceFactory } from "@/domains/service/models";

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "service-created", service: any): void;
}>();

const serviceDialogStep = ref(1);
const newServiceForm = ref({
  name: "",
  directory: "",
  isAuth: false,
  authType: "bearer",
  environments: [
    {
      name: "BASE_URL",
      dev: "http://localhost:3000",
      stage: "https://stage.api.com",
      prod: "https://api.com",
    },
  ],
});

const selectDirectory = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: "Select Service Directory",
    });
    if (selected) {
      return Array.isArray(selected) ? selected[0] : selected;
    }
  } catch (error) {
    console.error("Failed to open directory dialog:", error);
    toast.error("Failed to open directory dialog");
  }
  return null;
};

const handleSelectDirectory = async () => {
  const directory = await selectDirectory();
  if (directory) {
    newServiceForm.value.directory = directory;
  }
};

const handleNextStep = () => {
  if (serviceDialogStep.value === 1 && newServiceForm.value.name) {
    serviceDialogStep.value = 2;
  }
};

const handlePrevStep = () => {
  serviceDialogStep.value = 1;
};

const addEnvRow = () => {
  newServiceForm.value.environments.push({
    name: "",
    dev: "",
    stage: "",
    prod: "",
  });
};

const handleAddService = () => {
  if (!newServiceForm.value.name.trim()) return;

  // Use factory for ID and initial structure
  const tempService = ServiceFactory.createDefault(
    newServiceForm.value.name,
    newServiceForm.value.directory,
  );

  const newService: any = {
    ...tempService,
    isAuthenticated: newServiceForm.value.isAuth,
    authType: newServiceForm.value.isAuth
      ? newServiceForm.value.authType.toLowerCase()
      : "none",
    environments: [
      {
        name: "DEV",
        isUnsafe: false,
        variables: newServiceForm.value.environments.map((env) => ({
          name: env.name,
          value: env.dev,
        })),
      },
      {
        name: "STAGE",
        isUnsafe: false,
        variables: newServiceForm.value.environments.map((env) => ({
          name: env.name,
          value: env.stage,
        })),
      },
      {
        name: "PROD",
        isUnsafe: true,
        variables: newServiceForm.value.environments.map((env) => ({
          name: env.name,
          value: env.prod,
        })),
      },
    ],
  };

  emit("service-created", newService);
  emit("update:open", false);

  // Reset form
  newServiceForm.value = {
    name: "",
    directory: "",
    isAuth: false,
    authType: "bearer",
    environments: [
      {
        name: "BASE_URL",
        dev: "http://localhost:3000",
        stage: "https://stage.api.com",
        prod: "https://api.com",
      },
    ],
  };
  serviceDialogStep.value = 1;

  toast.success("Service Created", {
    description: `Service "${newService.name}" has been added to your workspace.`,
  });
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[700px]">
      <DialogHeader>
        <div class="flex items-center justify-between">
          <div>
            <DialogTitle>Add New Service</DialogTitle>
            <DialogDescription>
              {{
                serviceDialogStep === 1
                  ? "Enter basic information for your new service."
                  : "Configure environment variables."
              }}
            </DialogDescription>
          </div>
          <div class="flex items-center gap-1">
            <div
              :class="[
                'h-2 w-8 rounded-full transition-colors',
                serviceDialogStep === 1 ? 'bg-primary' : 'bg-muted',
              ]"
            ></div>
            <div
              :class="[
                'h-2 w-8 rounded-full transition-colors',
                serviceDialogStep === 2 ? 'bg-primary' : 'bg-muted',
              ]"
            ></div>
          </div>
        </div>
      </DialogHeader>

      <!-- Step 1: Basic Config -->
      <div v-if="serviceDialogStep === 1" class="grid gap-6 py-4">
        <div class="grid gap-2">
          <Label for="service-name" class="">Service Name</Label>
          <Input
            id="service-name"
            v-model="newServiceForm.name"
            placeholder="E.g. Payment Gateway"
            class="h-9"
          />
        </div>
        <div class="grid gap-2">
          <Label for="service-dir" class="">Save Directory</Label>
          <div class="flex gap-2">
            <Input
              id="service-dir"
              v-model="newServiceForm.directory"
              placeholder="Select directory or enter path..."
              class="h-9 flex-1"
            />
            <Button
              variant="outline"
              size="icon"
              class="h-9 w-9"
              @click="handleSelectDirectory"
            >
              <Folder class="h-4 w-4" />
            </Button>
          </div>
        </div>
        <div
          class="flex items-center justify-between py-2 border-b border-dashed pb-4"
        >
          <div class="space-y-0.5">
            <Label for="is-auth" class="font-semibold"
              >Service Authentication</Label
            >
            <div class="text-muted-foreground">
              Enable if this service requires a global auth token.
            </div>
          </div>
          <Switch id="is-auth" v-model:checked="newServiceForm.isAuth" />
        </div>
        <div
          v-if="newServiceForm.isAuth"
          class="grid gap-2 animate-in fade-in slide-in-from-top-2 pt-2 border-l-2 border-primary/20 pl-4 ml-1"
        >
          <Label for="auth-type" class="">Authentication Type</Label>
          <Select v-model="newServiceForm.authType">
            <SelectTrigger id="auth-type" class="h-9">
              <SelectValue placeholder="Select auth type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="bearer" class="">Bearer Token</SelectItem>
              <SelectItem value="basic" class="">Basic Auth</SelectItem>
              <SelectItem value="apikey" class="">API Key</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <!-- Step 2: Environments Table -->
      <div v-if="serviceDialogStep === 2" class="py-4">
        <div class="border rounded-md overflow-hidden">
          <Table>
            <TableHeader class="bg-muted/30">
              <TableRow>
                <TableHead class="w-[150px] font-bold">Key</TableHead>
                <TableHead class="font-bold">Dev</TableHead>
                <TableHead class="font-bold">Stage</TableHead>
                <TableHead class="font-bold">Prod</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="(env, idx) in newServiceForm.environments"
                :key="idx"
              >
                <TableCell class="p-2 border-r">
                  <Input
                    v-model="env.name"
                    class="h-8 border-none focus-visible:ring-0 shadow-none px-1"
                  />
                </TableCell>
                <TableCell class="p-2 border-r">
                  <Input
                    v-model="env.dev"
                    class="h-8 border-none focus-visible:ring-0 shadow-none px-1"
                  />
                </TableCell>
                <TableCell class="p-2 border-r">
                  <Input
                    v-model="env.stage"
                    class="h-8 border-none focus-visible:ring-0 shadow-none px-1"
                  />
                </TableCell>
                <TableCell class="p-2">
                  <Input
                    v-model="env.prod"
                    class="h-8 border-none focus-visible:ring-0 shadow-none px-1"
                  />
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
        <Button
          variant="ghost"
          size="sm"
          class="mt-2 h-7 gap-1 text-primary hover:bg-primary/5"
          @click="addEnvRow"
        >
          <Plus class="h-3 w-3" /> Add Variable
        </Button>
      </div>

      <DialogFooter class="gap-2">
        <Button
          v-if="serviceDialogStep === 1"
          variant="ghost"
          size="sm"
          @click="emit('update:open', false)"
          >Cancel</Button
        >
        <Button
          v-if="serviceDialogStep === 2"
          variant="outline"
          size="sm"
          @click="handlePrevStep"
        >
          <ChevronLeft class="mr-1 h-3 w-3" /> Back
        </Button>

        <Button
          v-if="serviceDialogStep === 1"
          size="sm"
          @click="handleNextStep"
          :disabled="!newServiceForm.name"
        >
          Next <ChevronRight class="ml-1 h-3 w-3" />
        </Button>
        <Button
          v-if="serviceDialogStep === 2"
          size="sm"
          @click="handleAddService"
        >
          Create Service
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
