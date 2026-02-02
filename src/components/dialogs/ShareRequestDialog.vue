<script setup lang="ts">
import { ref } from "vue";
import { Share2, Copy } from "lucide-vue-next";
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
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useCodeGenerator } from "@/composables/useCodeGenerator";

defineProps<{
  open: boolean;
  tab: any;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const { generateCodeSnippet } = useCodeGenerator();
const shareLanguage = ref("curl");

const copyToClipboard = (text: string) => {
  navigator.clipboard.writeText(text);
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[700px] p-0 overflow-hidden gap-0">
      <DialogHeader class="p-6 pb-4 border-b">
        <DialogTitle class="flex items-center gap-2">
          <Share2 class="h-5 w-5 text-primary" />
          Generate Code Snippet
        </DialogTitle>
        <DialogDescription>
          Export your request to various programming languages and formats.
        </DialogDescription>
      </DialogHeader>

      <div class="flex flex-col h-[450px]">
        <!-- Setup Bar -->
        <div class="p-4 bg-muted/20 border-b flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="font-medium text-muted-foreground"
              >Select Language:</span
            >
            <Select v-model="shareLanguage">
              <SelectTrigger class="w-40 h-8 font-semibold">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="curl" class=" ">cURL</SelectItem>
                <SelectItem value="node" class=" ">Node.js (Axios)</SelectItem>
                <SelectItem value="java" class=" "
                  >Java (HttpClient)</SelectItem
                >
                <SelectItem value="kotlin" class=" "
                  >Kotlin (OkHttp)</SelectItem
                >
              </SelectContent>
            </Select>
          </div>
          <Button
            size="sm"
            variant="outline"
            class="h-8 gap-2"
            @click="copyToClipboard(generateCodeSnippet(shareLanguage, tab))"
          >
            <Copy class="h-3.5 w-3.5" /> Copy Code
          </Button>
        </div>

        <!-- Code Display Area -->
        <div
          class="flex-1 overflow-auto bg-[#0a0a0a] p-6 leading-relaxed text-zinc-300 relative group"
        >
          <div
            class="absolute top-4 right-4 uppercase font-bold text-zinc-600 tracking-widest px-2 py-1 border border-zinc-800 rounded"
          >
            {{ shareLanguage }}
          </div>
          <pre class="whitespace-pre-wrap">{{
            generateCodeSnippet(shareLanguage, tab)
          }}</pre>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
