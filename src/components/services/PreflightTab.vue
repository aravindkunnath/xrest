<script setup lang="ts">
import { Globe, Play, Clock, Fingerprint } from "lucide-vue-next";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import InterpolatedInput from "../InterpolatedInput.vue";
import InterpolatedTextarea from "../InterpolatedTextarea.vue";
import RequestParameters from "../RequestParameters.vue";
import TestPreflightDialog from "@/components/TestPreflightDialog.vue";
import { computed, ref } from "vue";

const props = defineProps<{
  preflight: any;
  variables: Record<string, string>;
  environmentName: string;
  serviceId?: string;
}>();

const showTestDialog = ref(false);

const preflight = computed(() => props.preflight);
const variables = computed(() => props.variables);
const environmentName = computed(() => props.environmentName);
const serviceId = computed(() => props.serviceId);
</script>

<template>
  <div class="space-y-6 pb-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
    <div class="flex items-center justify-between border-b pb-4">
      <div class="flex items-center gap-2">
        <Globe class="h-5 w-5 text-primary" />
        <div>
          <h3 class="font-bold uppercase tracking-wider text-sm">
            Pre-flight Configuration
          </h3>
          <p class="text-xs text-muted-foreground">
            Configure how to automatically obtain authentication tokens.
          </p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2 px-3 py-1 bg-muted rounded-full border">
          <Label class="text-[10px] font-bold uppercase cursor-pointer" for="preflight-enable">Enabled</Label>
          <Switch id="preflight-enable" v-model="preflight.enabled" />
        </div>
        <button @click="showTestDialog = true"
          class="flex items-center gap-1.5 px-4 py-1.5 bg-primary text-primary-foreground rounded-md font-bold text-xs shadow-sm hover:opacity-90 transition-all active:scale-95">
          <Play class="h-3 w-3 fill-current" /> TEST SEQUENCE
        </button>
      </div>
    </div>

    <TestPreflightDialog v-if="serviceId" v-model:open="showTestDialog" :service-id="serviceId" :config="preflight"
      :variables="variables" />

    <div v-if="preflight.enabled" class="space-y-6 animate-in fade-in zoom-in-95 duration-300">
      <!-- Request Section -->
      <div class="space-y-4 bg-muted/20 p-4 rounded-xl border">
        <div class="flex items-center gap-2">
          <Fingerprint class="h-4 w-4 text-muted-foreground" />
          <Label class="font-bold uppercase text-xs tracking-tight">Request Details</Label>
        </div>
        
        <div class="flex items-center gap-2 bg-background p-2 rounded-lg border shadow-sm">
          <Select v-model="preflight.method">
            <SelectTrigger class="w-28 h-9 font-bold border-none shadow-none focus:ring-0">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="GET" class="font-bold text-green-600">GET</SelectItem>
              <SelectItem value="POST" class="font-bold text-orange-600">POST</SelectItem>
              <SelectItem value="PUT" class="font-bold text-blue-600">PUT</SelectItem>
            </SelectContent>
          </Select>
          <div class="w-px h-6 bg-border mx-1" />
          <InterpolatedInput v-model="preflight.url" :variables="variables" :environment-name="environmentName"
            placeholder="Auth URL (e.g. https://auth.api.com/oauth/token)" class="flex-1 h-9 border-none shadow-none focus-visible:ring-0" />
        </div>

        <div v-if="preflight.method !== 'GET'" class="space-y-2 pt-2">
          <div class="flex items-center justify-between">
            <Label class="text-xs font-bold uppercase text-muted-foreground">Payload</Label>
            <Select v-model="preflight.bodyType">
              <SelectTrigger class="h-7 w-fit font-bold text-[10px] uppercase bg-muted border-none shadow-none px-3 rounded-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="application/json">JSON</SelectItem>
                <SelectItem value="application/x-www-form-urlencoded">Form URL Encoded</SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <div v-if="preflight.bodyType === 'application/json'" class="border rounded-lg overflow-hidden bg-background">
            <InterpolatedTextarea v-model="preflight.body" :variables="variables" :environment-name="environmentName"
              language="json"
              class="w-full h-32 bg-transparent border-none p-3 resize-none focus:ring-0 font-mono text-sm"
              placeholder='{ "grant_type": "client_credentials", ... }' />
          </div>
          <div v-else class="bg-background rounded-lg border p-1">
            <RequestParameters :items="preflight.bodyParams" :variables="variables"
              :environment-name="environmentName" />
          </div>
        </div>
      </div>

      <!-- Extraction & Caching Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Token Extraction -->
        <div class="space-y-4 bg-muted/20 p-4 rounded-xl border">
          <div class="flex items-center gap-2">
            <Play class="h-4 w-4 text-muted-foreground" />
            <Label class="font-bold uppercase text-xs tracking-tight">Token Extraction</Label>
          </div>
          
          <div class="space-y-4">
            <div class="space-y-2">
              <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Path in Response JSON</Label>
              <Input v-model="preflight.tokenKey" class="h-9 bg-background font-mono text-xs" placeholder="access_token" />
            </div>
            <div class="space-y-2">
              <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Target Header Name</Label>
              <Input v-model="preflight.tokenHeader" class="h-9 bg-background font-mono text-xs" placeholder="Authorization" />
              <p class="text-[10px] text-muted-foreground italic px-1">
                If "Authorization", it will be used as a Bearer token.
              </p>
            </div>
          </div>
        </div>

        <!-- Caching -->
        <div class="space-y-4 bg-muted/20 p-4 rounded-xl border">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Clock class="h-4 w-4 text-muted-foreground" />
              <Label class="font-bold uppercase text-xs tracking-tight">Caching Strategy</Label>
            </div>
            <Switch v-model="preflight.cacheToken" />
          </div>

          <div v-if="preflight.cacheToken" class="space-y-4 animate-in slide-in-from-top-1 duration-200">
            <div class="space-y-2">
              <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Duration Source</Label>
              <Select v-model="preflight.cacheDurationMode">
                <SelectTrigger class="h-9 bg-background">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="derived">Extract from Response</SelectItem>
                  <SelectItem value="manual">Manual Entry</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div v-if="preflight.cacheDurationMode === 'derived'" class="grid grid-cols-2 gap-2">
              <div class="space-y-2">
                <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Key</Label>
                <Input v-model="preflight.cacheDurationKey" class="h-9 bg-background font-mono text-xs" placeholder="expires_in" />
              </div>
              <div class="space-y-2">
                <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Unit</Label>
                <Select v-model="preflight.cacheDurationUnit">
                  <SelectTrigger class="h-9 bg-background">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="seconds">Seconds</SelectItem>
                    <SelectItem value="minutes">Minutes</SelectItem>
                    <SelectItem value="hours">Hours</SelectItem>
                    <SelectItem value="days">Days</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div v-else class="space-y-2">
              <Label class="text-[10px] font-bold uppercase text-muted-foreground ml-1">Seconds until Expiry</Label>
              <Input type="number" v-model.number="preflight.cacheDurationSeconds" class="h-9 bg-background" placeholder="3600" />
            </div>
          </div>
          <div v-else class="h-full flex items-center justify-center border border-dashed rounded-lg bg-background/50">
            <p class="text-[10px] text-muted-foreground uppercase font-medium">Caching Disabled</p>
          </div>
        </div>
      </div>

      <div class="text-[11px] text-muted-foreground italic bg-primary/5 p-4 rounded-xl border border-primary/10 leading-relaxed">
        <strong>Tip:</strong> The pre-flight request runs before your main request only if the token is missing or expired.
        It inherits the same environment variables as your service.
      </div>
    </div>
    
    <div v-else class="h-[400px] flex flex-col items-center justify-center border border-dashed rounded-3xl bg-muted/5 gap-4">
      <div class="h-16 w-16 bg-muted/20 rounded-full flex items-center justify-center border border-muted-foreground/10">
        <Globe class="h-8 w-8 text-muted-foreground/40" />
      </div>
      <div class="text-center">
        <h4 class="font-bold text-muted-foreground uppercase text-xs tracking-widest mb-1">Pre-flight is Disabled</h4>
        <p class="text-[10px] text-muted-foreground/60 max-w-[200px]">
          Enable it to automatically fetch and refresh authentication tokens from an external provider.
        </p>
      </div>
      <button @click="preflight.enabled = true" 
        class="mt-2 text-xs font-bold uppercase px-6 py-2 bg-background border rounded-full hover:bg-muted transition-colors shadow-sm">
        Enable Sequence
      </button>
    </div>
  </div>
</template>
