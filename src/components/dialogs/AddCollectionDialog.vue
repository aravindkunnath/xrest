<script setup lang="ts">
import { ref } from "vue";
import { toast } from "vue-sonner";
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
  (e: "collection-created", collection: any): void;
}>();

const newCollectionForm = ref({
  name: "",
});

const handleAddCollection = () => {
  if (!newCollectionForm.value.name.trim()) return;

  const newCollection: any = {
    id: `c-${Date.now()}`,
    name: newCollectionForm.value.name,
    directory: "", // Collections don't have a linked directory
    isAuthenticated: false,
    authType: "none",
    endpoints: [],
    environments: [
      {
        name: "GLOBAL",
        isUnsafe: false,
        variables: [{ name: "BASE_URL", value: "https://api.example.com" }],
      },
    ],
    selectedEnvironment: "GLOBAL",
  };

  emit("collection-created", newCollection);
  emit("update:open", false);

  // Reset form
  newCollectionForm.value = {
    name: "",
  };

  toast.success("Collection Created", {
    description: `Collection "${newCollection.name}" has been added to your scratchpad.`,
  });
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[425px]">
      <DialogHeader>
        <DialogTitle>New Collection</DialogTitle>
        <DialogDescription>
          Create a new API scratchpad collection.
        </DialogDescription>
      </DialogHeader>

      <div class="grid gap-4 py-4">
        <div class="grid gap-2">
          <Label for="collection-name" class="text-muted-foreground mr-auto"
            >Collection Name</Label
          >
          <Input
            id="collection-name"
            v-model="newCollectionForm.name"
            placeholder="E.g. My API Tests"
            class="h-9"
            @keyup.enter="handleAddCollection"
          />
        </div>
      </div>

      <DialogFooter>
        <Button variant="ghost" size="sm" @click="emit('update:open', false)"
          >Cancel</Button
        >
        <Button
          size="sm"
          @click="handleAddCollection"
          :disabled="!newCollectionForm.name"
        >
          Create Collection
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
