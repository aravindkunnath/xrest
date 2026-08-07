<script setup lang="ts">
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Check, Copy, Trash2, Wand2 } from "@lucide/vue";
import { computed, ref } from "vue";
import { toast } from "vue-sonner";
import CodeMirrorEditor from "@/components/editor/CodeMirrorEditor.vue";

const props = defineProps<{
  body: any;
  variables: Record<string, string>;
  environmentName: string;
}>();

const emit = defineEmits<{
  (e: "update:content", value: string): void;
}>();

const body = computed(() => props.body);
const variables = computed(() => props.variables);
const environmentName = computed(() => props.environmentName);

const isCopied = ref(false);

const handleContentUpdate = (val: string) => {
  if (body.value) {
    body.value.content = val;
  }
  emit("update:content", val);
};

const prettifyJson = () => {
  if (!body.value?.content) return;
  try {
    const rawContent = body.value.content.trim();
    if (!rawContent) return;

    // Replace {{variable}} with placeholder strings so JSON.parse doesn't throw syntax errors on unquoted templates
    const placeholders: string[] = [];
    const sanitized = rawContent.replace(/\{\{\s*[\w.-]+\s*\}\}/g, (match: string) => {
      const token = `__XREST_VAR_${placeholders.length}__`;
      placeholders.push(match);
      return `"${token}"`;
    });

    const parsed = JSON.parse(sanitized);
    let formatted = JSON.stringify(parsed, null, 2);

    // Restore original {{variable}} placeholders (unquoting them if they were original template tokens)
    placeholders.forEach((originalVar, idx) => {
      const token = `__XREST_VAR_${idx}__`;
      formatted = formatted.replace(new RegExp(`"${token}"`, "g"), originalVar);
      formatted = formatted.replace(new RegExp(token, "g"), originalVar);
    });

    handleContentUpdate(formatted);
    toast.success("JSON formatted successfully");
  } catch (err: any) {
    toast.error("Invalid JSON content: Unable to format");
  }
};

const clearBody = () => {
  handleContentUpdate("");
};

const copyBody = async () => {
  if (!body.value?.content) return;
  try {
    await navigator.clipboard.writeText(body.value.content);
    isCopied.value = true;
    setTimeout(() => {
      isCopied.value = false;
    }, 1500);
    toast.success("Body copied to clipboard");
  } catch (err) {
    toast.error("Failed to copy body");
  }
};
</script>

<template>
  <div class="space-y-3 p-4 h-full flex flex-col">
    <!-- Header Toolbar -->
    <div class="flex items-center justify-between shrink-0">
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-foreground/80 tracking-tight">
            Content-Type:
          </span>
          <Select v-model="body.type">
            <SelectTrigger class="w-52 h-7 text-xs font-mono">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="application/json" class="text-xs font-mono">
                JSON (application/json)
              </SelectItem>
              <SelectItem value="application/xml" class="text-xs font-mono">
                XML (application/xml)
              </SelectItem>
              <SelectItem value="text/plain" class="text-xs font-mono">
                Text (text/plain)
              </SelectItem>
              <SelectItem value="application/x-www-form-urlencoded" class="text-xs font-mono">
                Form URL Encoded
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <Button v-if="body.type === 'application/json'" variant="ghost" size="sm"
          class="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground" @click="prettifyJson"
          title="Format JSON payload">
          <Wand2 class="h-3.5 w-3.5 text-primary" />
          <span class="hidden sm:inline">Prettify</span>
        </Button>

        <Button variant="ghost" size="sm" class="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground"
          @click="copyBody" title="Copy payload to clipboard">
          <component :is="isCopied ? Check : Copy" class="h-3.5 w-3.5" />
          <span class="hidden sm:inline">{{ isCopied ? "Copied" : "Copy" }}</span>
        </Button>

        <Button variant="ghost" size="sm" class="h-7 text-xs px-2 text-muted-foreground hover:text-destructive gap-1"
          @click="clearBody" title="Clear request body">
          <Trash2 class="h-3.5 w-3.5" />
          <span class="hidden sm:inline">Clear</span>
        </Button>
      </div>
    </div>

    <!-- Body CodeMirror Editor Container -->
    <div class="relative group flex-1 min-h-[200px] flex flex-col overflow-hidden">
      <CodeMirrorEditor
        :model-value="body.content"
        @update:model-value="handleContentUpdate"
        :variables="variables"
        :environment-name="environmentName"
        :language="body.type"
        class="w-full h-full min-h-[200px]"
        placeholder="Enter request body payload here..."
      />
      <div class="absolute bottom-3 right-3 opacity-40 group-hover:opacity-100 transition-opacity pointer-events-none z-30">
        <span class="px-1.5 py-0.5 rounded bg-muted/80 border text-[10px] font-mono text-muted-foreground uppercase">
          {{ body.type?.split('/')[1] || body.type }}
        </span>
      </div>
    </div>
  </div>
</template>
