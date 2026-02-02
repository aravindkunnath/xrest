<script setup lang="ts">
import { ShieldCheck, Globe, Play } from "lucide-vue-next";
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
import InterpolatedInput from "./InterpolatedInput.vue";
import InterpolatedTextarea from "./InterpolatedTextarea.vue";
import RequestParameters from "./RequestParameters.vue";
import TestPreflightDialog from "@/components/TestPreflightDialog.vue";
import { computed, ref } from "vue";

const props = defineProps<{
  auth: any;
  preflight: any;
  variables: Record<string, string>;
  environmentName: string;
  serviceId?: string;
}>();

const showTestDialog = ref(false);

const auth = computed(() => props.auth);
const preflight = computed(() => props.preflight);
const variables = computed(() => props.variables);
const environmentName = computed(() => props.environmentName);
const serviceId = computed(() => props.serviceId);
</script>

<template>
  <div class="space-y-8 pb-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
    <!-- Auth Configuration -->
    <div class="space-y-4">
      <div class="flex items-center gap-2 border-b pb-2">
        <ShieldCheck class="h-4 w-4 text-primary" />
        <h3 class="font-semibold uppercase tracking-wider">
          Authentication Configuration
        </h3>
      </div>

      <div class="grid grid-cols-[120px_1fr] items-center gap-4">
        <Label class="text-muted-foreground">Auth Type</Label>
        <Select v-model="auth.type">
          <SelectTrigger class="h-8 w-48 font-medium">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="bearer" class="">Bearer Token</SelectItem>
            <SelectItem value="basic" class="">Basic Auth</SelectItem>
            <SelectItem value="apikey" class="">API Key</SelectItem>
            <SelectItem value="none" class="">No Auth</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- Bearer Config -->
      <div v-if="auth.type === 'bearer'"
        class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
        <Label class="text-muted-foreground">Token</Label>
        <InterpolatedInput v-model="auth.bearerToken" :variables="variables" :environment-name="environmentName"
          placeholder="eyJhbGciOiJIUzI1..." class="h-8 bg-muted/20" />
      </div>

      <!-- Basic Auth Config -->
      <template v-if="auth.type === 'basic'">
        <div
          class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
          <Label class="text-muted-foreground">Username</Label>
          <InterpolatedInput v-model="auth.basicUser" :variables="variables" :environment-name="environmentName"
            placeholder="Username" class="h-8 bg-muted/20" />
        </div>
        <div
          class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
          <Label class="text-muted-foreground">Password</Label>
          <InterpolatedInput v-model="auth.basicPass" :variables="variables" :environment-name="environmentName"
            placeholder="Password" class="h-8 bg-muted/20" />
        </div>
      </template>

      <!-- API Key Config -->
      <template v-if="auth.type === 'apikey'">
        <div
          class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
          <Label class="text-muted-foreground">Key Name</Label>
          <InterpolatedInput v-model="auth.apiKeyName" :variables="variables" :environment-name="environmentName"
            placeholder="X-API-Key" class="h-8 bg-muted/20" />
        </div>
        <div
          class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
          <Label class="text-muted-foreground">Key Value</Label>
          <InterpolatedInput v-model="auth.apiKeyValue" :variables="variables" :environment-name="environmentName"
            placeholder="Value" class="h-8 bg-muted/20" />
        </div>
        <div
          class="grid grid-cols-[120px_1fr] items-center gap-4 animate-in fade-in slide-in-from-left-2 transition-all">
          <Label class="text-muted-foreground">Add to</Label>
          <Select v-model="auth.apiKeyLocation">
            <SelectTrigger class="h-8 w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="header" class="">Header</SelectItem>
              <SelectItem value="query" class="">Query Params</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </template>
    </div>

    <div class="space-y-4 pt-4">
      <div class="flex items-center gap-2 border-b pb-2">
        <Switch v-model="preflight.enabled" />
        <Globe class="h-4 w-4 text-primary" />
        <h3 class="font-semibold uppercase tracking-wider">
          Authentication Pre-flight
        </h3>
        <button @click="showTestDialog = true"
          class="ml-auto flex items-center gap-1.5 px-3 py-1 bg-primary/10 text-primary rounded-full font-bold border border-primary/20 hover:bg-primary/20 transition-all">
          <Play class="h-2.5 w-2.5 fill-primary" /> TEST SEQUENCE
        </button>
      </div>

      <TestPreflightDialog v-if="serviceId" v-model:open="showTestDialog" :service-id="serviceId" :config="preflight"
        :variables="variables" />

      <div v-if="preflight.enabled"
        class="bg-muted/30 border rounded-lg overflow-hidden space-y-4 p-4 animate-in fade-in slide-in-from-top-2">
        <div class="space-y-3">
          <Label class="font-bold uppercase text-muted-foreground">Request Details</Label>
          <div class="flex items-center gap-2 bg-muted/50 p-2 rounded border">
            <Select v-model="preflight.method">
              <SelectTrigger class="w-24 h-8 font-bold bg-background">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="GET" class="font-bold text-green-600">GET</SelectItem>
                <SelectItem value="POST" class="font-bold text-orange-600">POST</SelectItem>
                <SelectItem value="PUT" class="font-bold text-blue-600">PUT</SelectItem>
              </SelectContent>
            </Select>
            <InterpolatedInput v-model="preflight.url" :variables="variables" :environment-name="environmentName"
              placeholder="Pre-flight URL (e.g. /oauth/token)" class="flex-1 h-8" />
          </div>

          <div v-if="preflight.method !== 'GET'" class="space-y-2">
            <div class="flex items-center justify-between">
              <Label class="text-muted-foreground uppercase font-bold tracking-tight">Request Body</Label>
              <Select v-model="preflight.bodyType">
                <SelectTrigger class="h-6 w-fit font-medium bg-background border-none shadow-none focus:ring-0 px-2">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="application/json" class="">JSON</SelectItem>
                  <SelectItem value="application/x-www-form-urlencoded" class="">URL Encoded</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div v-show="preflight.bodyType === 'application/json'">
              <InterpolatedTextarea v-model="preflight.body" :variables="variables" :environment-name="environmentName"
                language="json"
                class="w-full h-24 bg-background border rounded p-2 resize-none focus:ring-1 focus:ring-primary"
                placeholder='{ "grant_type": "password", ... }' />
            </div>
            <div v-show="preflight.bodyType === 'application/x-www-form-urlencoded'
              ">
              <RequestParameters :items="preflight.bodyParams" :variables="variables"
                :environment-name="environmentName" />
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4 pt-2 border-t mt-2">
          <div class="space-y-2">
            <Label class="font-bold uppercase text-muted-foreground">Token Extraction</Label>
            <div class="space-y-1.5">
              <Label class="text-muted-foreground">Token Key (Response JSON)</Label>
              <Input v-model="preflight.tokenKey" class="h-7" placeholder="access_token" />
            </div>
            <div class="space-y-1.5">
              <Label class="text-muted-foreground">Token Header Name</Label>
              <Input v-model="preflight.tokenHeader" class="h-7" placeholder="Authorization" />
            </div>
          </div>

          <div class="space-y-2">
            <Label class="font-bold uppercase text-muted-foreground">Caching & Expiration</Label>
            <div class="flex items-center justify-between mb-1">
              <Label class="text-muted-foreground">Enable Caching</Label>
              <Switch v-model="preflight.cacheToken" class="scale-75" />
            </div>
            <div v-if="preflight.cacheToken" class="space-y-2 animate-in fade-in">
              <div class="space-y-1.5">
                <Label class="text-muted-foreground">Duration Key / Unit</Label>
                <div class="flex gap-2">
                  <Input v-model="preflight.cacheDurationKey" class="h-7 flex-1" placeholder="expires_in" />
                  <Select v-model="preflight.cacheDurationUnit">
                    <SelectTrigger class="h-7 w-20">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="seconds" class="">sec</SelectItem>
                      <SelectItem value="minutes" class="">min</SelectItem>
                      <SelectItem value="hours" class="">hrs</SelectItem>
                      <SelectItem value="days" class="">days</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="text-muted-foreground italic bg-primary/5 p-2 rounded border border-primary/10">
          // This request will run automatically to obtain and refresh the
          authentication token.
        </div>
      </div>
    </div>
  </div>
</template>
