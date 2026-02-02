<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Save, Share2, ShieldCheck } from "lucide-vue-next";
import InterpolatedInput from "./InterpolatedInput.vue";

defineProps<{
  method: string;
  url: string;
  isSending: boolean;
  isUnsafe: boolean;
  variables: Record<string, string>;
  environmentName: string;
}>();

const emit = defineEmits<{
  (e: "update:method", value: string): void;
  (e: "update:url", value: string): void;
  (e: "send"): void;
  (e: "save"): void;
  (e: "share"): void;
}>();

const getMethodColor = (m: string) => {
  switch (m?.toUpperCase()) {
    case "GET":
      return "text-green-600";
    case "POST":
      return "text-orange-600";
    case "PUT":
      return "text-blue-600";
    case "DELETE":
      return "text-red-600";
    case "PATCH":
      return "text-purple-600";
    default:
      return "text-muted-foreground";
  }
};
</script>

<template>
  <div class="flex items-center gap-2 mb-4">
    <Select
      :model-value="method"
      @update:model-value="emit('update:method', $event as string)"
    >
      <SelectTrigger class="w-24 h-9 font-bold bg-muted border-none gap-2">
        <SelectValue>
          <span :class="getMethodColor(method)">{{ method }}</span>
        </SelectValue>
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="GET" class="font-bold text-green-600"
          >GET</SelectItem
        >
        <SelectItem value="POST" class="font-bold text-orange-600"
          >POST</SelectItem
        >
        <SelectItem value="PUT" class="font-bold text-blue-600">PUT</SelectItem>
        <SelectItem value="DELETE" class="font-bold text-red-600"
          >DELETE</SelectItem
        >
        <SelectItem value="PATCH" class="font-bold text-purple-600"
          >PATCH</SelectItem
        >
      </SelectContent>
    </Select>

    <InterpolatedInput
      :key="Object.keys(variables || {}).length ? 'vars-loaded' : 'vars-empty'"
      :model-value="url"
      @update:model-value="emit('update:url', $event)"
      :variables="variables"
      :environment-name="environmentName"
      class="flex-1 h-9 focus-visible:ring-1 focus-visible:ring-primary"
      placeholder="Enter URL"
    />

    <div class="flex items-center gap-1 shrink-0">
      <button
        @click="emit('send')"
        :disabled="isSending"
        :class="[
          'h-9 px-4 rounded-md font-medium transition-colors disabled:opacity-50 flex items-center gap-2 shadow-sm',
          isUnsafe
            ? 'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-destructive/20'
            : 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-primary/10',
        ]"
      >
        <span
          v-if="isSending"
          class="h-3 w-3 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin"
        ></span>
        {{ isSending ? "Sending..." : "Send" }}
        <ShieldCheck v-if="isUnsafe" class="h-3.5 w-3.5 ml-0.5 opacity-90" />
      </button>

      <button
        @click="emit('save')"
        class="p-2 border rounded-md hover:bg-muted transition-colors text-muted-foreground"
        title="Save Request"
      >
        <Save class="h-4 w-4" />
      </button>

      <button
        @click="emit('share')"
        class="p-2 border rounded-md hover:bg-muted transition-colors text-muted-foreground"
        title="Share Request"
      >
        <Share2 class="h-4 w-4" />
      </button>
    </div>
  </div>
</template>
