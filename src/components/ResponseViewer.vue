<script setup lang="ts">
import { Copy, X } from "lucide-vue-next";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import JsonHighlighter from "./JsonHighlighter.vue";
import InterpolatedText from "./InterpolatedText.vue";
import { toast } from "vue-sonner";
import { computed } from "vue";

const props = defineProps<{
  response: {
    activeTab: string;
    status: number;
    statusText: string;
    time: string;
    size: string;
    type: string;
    body: string;
    error: string;
    headers: any[];
    requestHeaders: any[];
  };
  url: string;
  variables: Record<string, string>;
  environmentName: string;
}>();

const response = computed(() => props.response);

const copyToClipboard = (text: string) => {
  navigator.clipboard.writeText(text);
  toast.success("Copied to clipboard", {
    duration: 2000,
  });
};
</script>

<template>
  <div class="h-full flex flex-col bg-muted/5">
    <div
      class="flex items-center justify-between px-4 py-2 border-b bg-muted/20"
    >
      <div class="flex gap-4">
        <button
          @click="response.activeTab = 'body'"
          :class="[
            ' font-semibold uppercase tracking-wider pb-1 transition-colors border-b-2',
            response.activeTab === 'body'
              ? 'text-primary border-primary'
              : 'text-muted-foreground border-transparent hover:text-foreground',
          ]"
        >
          Body
        </button>
        <button
          @click="response.activeTab = 'headers'"
          :class="[
            ' font-semibold uppercase tracking-wider pb-1 transition-colors border-b-2',
            response.activeTab === 'headers'
              ? 'text-primary border-primary'
              : 'text-muted-foreground border-transparent hover:text-foreground',
          ]"
        >
          Headers
        </button>
      </div>
      <div class="flex gap-4">
        <span
          :class="[
            response.status >= 200 && response.status < 300
              ? 'text-green-600'
              : 'text-destructive',
            'font-bold',
          ]"
        >
          {{
            response.status
              ? `${response.status} ${response.statusText}`
              : response.statusText
          }}
        </span>
        <span class="text-muted-foreground">{{ response.time }}</span>
        <span class="text-muted-foreground">{{ response.size }}</span>
      </div>
    </div>

    <div class="flex-1 overflow-auto p-4">
      <!-- Error Display -->
      <div
        v-if="response.error"
        class="mb-4 p-4 border border-destructive/20 bg-destructive/5 rounded-md animate-in fade-in slide-in-from-top-2 duration-300"
      >
        <div class="flex items-start gap-3">
          <X class="h-4 w-4 text-destructive mt-0.5" />
          <div class="space-y-1">
            <h4 class="font-bold text-destructive">Request Failed</h4>
            <p class="text-destructive/80 break-all">{{ response.error }}</p>
          </div>
        </div>
      </div>

      <!-- Response Body Tab -->
      <div
        v-if="response.activeTab === 'body'"
        class="h-full flex flex-col gap-3 animate-in fade-in duration-300"
      >
        <div class="flex items-center justify-between">
          <span
            class="text-muted-foreground font-medium tracking-tight"
          >
            Content-Type:
            <span class="text-foreground ml-1">{{ response.headers.find((h: any) => h.name.toLowerCase() === 'content-type')?.value }}</span>
          </span>
          <button
            @click="copyToClipboard(response.body)"
            class="flex items-center gap-1.5 px-2 py-1 hover:bg-muted rounded text-muted-foreground transition-colors"
          >
            <Copy class="h-3 w-3" /> Copy
          </button>
        </div>
        <JsonHighlighter :code="response.body" class="flex-1 min-h-[100px]" />
      </div>

      <!-- Response Headers Tab -->
      <div
        v-if="response.activeTab === 'headers'"
        class="animate-in fade-in duration-300 pb-4"
      >
        <Accordion
          type="multiple"
          class="w-full"
          :default-value="['response-headers']"
        >
          <AccordionItem
            value="response-headers"
            class="border rounded-md px-3 mb-3 bg-background shadow-xs"
          >
            <AccordionTrigger class="py-2 hover:no-underline">
              <span class="font-bold uppercase text-primary"
                >Response Headers</span
              >
            </AccordionTrigger>
            <AccordionContent>
              <div class="space-y-1 py-1">
                <div
                  v-for="h in response.headers"
                  :key="h.name"
                  class="grid grid-cols-[140px_1fr] gap-4 border-b border-dashed border-muted pb-1 last:border-0"
                >
                  <span class="font-medium text-muted-foreground">{{
                    h.name
                  }}</span>
                  <span class="truncate">{{ h.value }}</span>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>

          <AccordionItem
            value="request-details"
            class="border rounded-md px-3 bg-background shadow-xs"
          >
            <AccordionTrigger class="py-2 hover:no-underline">
              <span class="font-bold uppercase text-muted-foreground"
                >Original Request</span
              >
            </AccordionTrigger>
            <AccordionContent>
              <div class="space-y-3 py-2">
                <div class="space-y-1">
                  <span
                    class="font-bold text-muted-foreground uppercase opacity-70"
                    >Target URL</span
                  >
                  <div
                    class="bg-muted/30 p-2 rounded break-all border group relative"
                  >
                    <InterpolatedText
                      :text="url"
                      :variables="variables"
                      :environment-name="environmentName"
                    />
                    <button
                      @click="copyToClipboard(url)"
                      class="absolute right-2 top-2 p-1 hover:bg-background rounded opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      <Copy class="h-3 w-3" />
                    </button>
                  </div>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </div>
    </div>
  </div>
</template>
