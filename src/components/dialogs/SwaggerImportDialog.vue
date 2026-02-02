<script setup lang="ts">
import { ref } from "vue";
import { toast } from "vue-sonner";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { Folder } from "lucide-vue-next";
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

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "import-complete"): void;
}>();

const swaggerForm = ref({
  url: "",
  file: "",
  name: "",
  directory: "",
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

const selectFile = async () => {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [
        {
          name: "Swagger/OpenAPI",
          extensions: ["json", "yaml", "yml"],
        },
      ],
    });
    if (selected) {
      swaggerForm.value.file = Array.isArray(selected) ? selected[0] : selected;
    }
  } catch (error) {
    console.error("Failed to open file dialog:", error);
    toast.error("Failed to open file dialog");
  }
};

const handleImportSwagger = async () => {
  if (!swaggerForm.value.name.trim()) {
    toast.error("Please enter a service name");
    return;
  }
  if (!swaggerForm.value.directory.trim()) {
    toast.error("Please select a save directory");
    return;
  }
  if (!swaggerForm.value.url.trim() && !swaggerForm.value.file.trim()) {
    toast.error("Please provide either a Swagger URL or a file");
    return;
  }

  try {
    const service: any = await invoke("import_swagger", {
      name: swaggerForm.value.name,
      directory: swaggerForm.value.directory,
      url: swaggerForm.value.url || null,
      file: swaggerForm.value.file || null,
    });

    emit("import-complete");
    emit("update:open", false);

    toast.success("Swagger Imported", {
      description: `Service "${service.name}" has been created from Swagger doc.`,
    });

    // Reset form
    swaggerForm.value = { url: "", file: "", name: "", directory: "" };
  } catch (error) {
    console.error("Failed to import Swagger:", error);
    toast.error("Import Failed", {
      description: String(error),
    });
  }
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[500px]">
      <DialogHeader>
        <DialogTitle>Import from Swagger/OpenAPI</DialogTitle>
        <DialogDescription>
          Provide a URL or upload a JSON/YAML file to create a new service.
        </DialogDescription>
      </DialogHeader>
      <div class="grid gap-6 py-4">
        <div class="grid gap-2">
          <Label for="swagger-name" class="">Service Name</Label>
          <Input
            id="swagger-name"
            v-model="swaggerForm.name"
            placeholder="E.g. My API Service"
            class="h-9"
          />
        </div>

        <div class="grid gap-2">
          <Label for="swagger-dir" class="">Save Directory</Label>
          <div class="flex gap-2">
            <Input
              id="swagger-dir"
              v-model="swaggerForm.directory"
              placeholder="Select directory to save service..."
              class="h-9 flex-1"
            />
            <Button
              variant="outline"
              size="icon"
              class="h-9 w-9"
              @click="
                async () => {
                  const dir = await selectDirectory();
                  if (dir) swaggerForm.directory = dir;
                }
              "
            >
              <Folder class="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div class="space-y-4 pt-4 border-t">
          <div class="grid gap-2">
            <Label for="swagger-url" class="">Swagger URL</Label>
            <Input
              id="swagger-url"
              v-model="swaggerForm.url"
              placeholder="https://api.example.com/swagger.json"
              class="h-9"
            />
          </div>

          <div class="relative">
            <div class="absolute inset-0 flex items-center">
              <span class="w-full border-t"></span>
            </div>
            <div class="relative flex justify-center uppercase">
              <span class="bg-background px-2 text-muted-foreground">Or</span>
            </div>
          </div>

          <div class="grid gap-2">
            <Label class="">Upload Swagger File</Label>
            <div class="flex gap-2">
              <Input
                :value="
                  swaggerForm.file
                    ? swaggerForm.file.split('/').pop() || swaggerForm.file
                    : ''
                "
                readonly
                placeholder="No file selected"
                class="h-9 flex-1 bg-muted/30"
              />
              <Button
                variant="outline"
                size="sm"
                class="h-9"
                @click="selectFile"
              >
                Browse
              </Button>
            </div>
          </div>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" size="sm" @click="emit('update:open', false)"
          >Cancel</Button
        >
        <Button
          size="sm"
          @click="handleImportSwagger"
          :disabled="
            !swaggerForm.name ||
            !swaggerForm.directory ||
            (!swaggerForm.url && !swaggerForm.file)
          "
        >
          Import Service
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
