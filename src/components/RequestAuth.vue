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
const variables = computed(() => props.variables);
const environmentName = computed(() => props.environmentName);
</script>

<template>
  <div class="space-y-8 pb-8 animate-in fade-in slide-in-from-bottom-2 duration-300">
    <!-- Auth Configuration -->
    <div class="space-y-4">
      <div class="flex items-center gap-2 border-b pb-2">
        <ShieldCheck class="h-4 w-4 text-primary" />
        <h3 class="font-semibold uppercase tracking-wider text-sm">
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
  </div>
</template>
