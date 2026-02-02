<script setup lang="ts">
import RequestAuth from "@/components/RequestAuth.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  addVariableToAll,
  getUniqueVariableNames,
  removeVariable,
  syncVariableName,
  syncVariableSecret,
  syncVariableValue,
  type Variable,
  type EnvironmentConfig,
} from "@/lib/environment-utils";
import { useSecretsStore } from "@/stores/secrets";
import {
  AlertCircle,
  Check,
  CheckCircle2,
  GitBranch,
  Globe,
  Key,
  Lock,
  Plus,
  RefreshCw,
  Save,
  Settings2,
  ShieldCheck,
  Trash2,
  Unlock
} from "lucide-vue-next";
import { useGitIntegration } from "@/composables/useGitIntegration";
import { useServiceSettings } from "@/composables/useServiceSettings";
import { computed, onMounted } from "vue";
import Switch from "./ui/switch/Switch.vue";

const props = defineProps<{
  tab: {
    id: string;
    type: string;
    serviceId: string;
    serviceData: any; // We could use Service type but it's dynamic
    title: string;
  };
  gitStatus?: any;
}>();

const tab = computed(() => props.tab);
const gitStatus = computed(() => props.gitStatus);
const secretsStore = useSecretsStore();
const {
  fetchGitStatus,
  handleCommitGit,
  handlePullGit,
  handlePushGit,
} = useGitIntegration();
const { saveSettings, deleteItem, reloadAll, initGit } =
  useServiceSettings();

onMounted(async () => {
  secretsStore.fetchSecrets();
  if (props.tab.serviceData.directory) {
    await fetchGitStatus(props.tab.serviceId, props.tab.serviceData.directory);
  }
});

const getVariable = (env: EnvironmentConfig, varName: string) => {
  if (!env.variables) return null;
  return env.variables.find((v: Variable) => v.name === varName);
};

const handleGitCommit = async () => {
  const message = prompt("Enter commit message", "Update service configuration");
  if (message) {
    await handleCommitGit(props.tab.serviceId, props.tab.serviceData.directory, message);
  }
};
</script>

<template>
  <div class="h-full overflow-auto p-6 max-w-5xl mx-auto space-y-8 pb-32">
    <!-- Header Area -->
    <div class="flex items-center justify-between border-b pb-4">
      <div class="space-y-1">
        <h2 class="text-xl font-bold tracking-tight">
          {{
            tab.serviceData.directory
              ? "Service Settings"
              : "Collection Settings"
          }}
        </h2>
        <p class="text-muted-foreground">
          Manage environments, variables, and
          {{
            tab.serviceData.directory ? "project integration" : "settings"
          }}
          for {{ tab.serviceData.name }}.
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Button v-if="tab.serviceData.directory" variant="outline" size="sm" class="h-8 gap-2"
          @click="reloadAll()">
          <RefreshCw class="h-3.5 w-3.5" /> Reload
        </Button>
        <Button variant="destructive" size="sm" class="h-8 gap-2" @click="deleteItem(tab.serviceId)">
          <Trash2 class="h-3.5 w-3.5" />
          {{
            tab.serviceData.directory ? "Delete Service" : "Delete Collection"
          }}
        </Button>
        <Button variant="default" size="sm" class="h-8 gap-2 px-4" @click="saveSettings(tab)">
          <Save class="h-3.5 w-3.5" /> Save Changes
        </Button>
      </div>
    </div>

    <!-- Main Content Stack -->
    <div class="space-y-6">

      <!-- General Settings Section -->
      <div class="bg-card rounded-lg border shadow-sm p-5 space-y-4">
        <div class="flex items-center gap-2 mb-2">
          <Settings2 class="h-4 w-4 text-primary" />
          <h3 class="font-semibold uppercase tracking-wider">
            General Information
          </h3>
        </div>
        <div class="grid gap-4">
          <div class="space-y-2">
            <Label class="">Service Name</Label>
            <Input v-model="tab.serviceData.name" class="h-9" placeholder="Enter service name" />
          </div>
          <div v-if="tab.serviceData.directory" class="space-y-2">
            <Label class="">Base Directory</Label>
            <div class="flex gap-2">
              <Input :model-value="tab.serviceData.directory" readonly class="h-9 bg-muted/30 flex-1" />
            </div>
            <p class="text-muted-foreground">
              All service configuration is stored in this directory as YAML
              files.
            </p>
          </div>
        </div>
      </div>

      <!-- Authentication Settings -->
      <div class="bg-card rounded-lg border shadow-sm p-5 space-y-4">
        <div class="flex items-center gap-2 mb-2">
          <ShieldCheck class="h-4 w-4 text-primary" />
          <h3 class="font-semibold uppercase tracking-wider">
            Service Authentication
          </h3>
        </div>
        <p class="text-sm text-muted-foreground">
          Configure authentication that will be used by default for all requests in this service.
        </p>
        <RequestAuth v-if="tab.serviceData.auth && tab.serviceData.preflight" v-model:auth="tab.serviceData.auth"
          v-model:preflight="tab.serviceData.preflight" :variables="{}" :environment-name="''"
          :service-id="tab.serviceId" />
      </div>

      <!-- Git Status Section -->
      <div v-if="tab.serviceData.directory" class="bg-card rounded-lg border shadow-sm p-5 space-y-4">
        <div class="flex items-center gap-2 mb-4">
          <GitBranch class="h-4 w-4 text-primary" />
          <h3 class="font-semibold uppercase tracking-wider">Git Status</h3>
        </div>
        <div v-if="gitStatus" class="space-y-4 mt-2">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground">Status</span>
            <div v-if="gitStatus.isGit"
              class="flex items-center gap-1.5 font-bold text-green-600 bg-green-50 px-2 py-0.5 rounded-full border border-green-100">
              <CheckCircle2 class="h-3 w-3" /> TRACKED
            </div>
            <div v-else
              class="flex items-center gap-1.5 font-bold text-muted-foreground bg-muted px-2 py-0.5 rounded-full border">
              UNTRACKED
            </div>
          </div>
          <div v-if="gitStatus.isGit" class="space-y-4">
            <div class="flex flex-col gap-3">
              <div class="flex items-center justify-between border-b border-dashed pb-2">
                <span class="text-muted-foreground">Branch</span>
                <span class="font-bold">{{ gitStatus.branch || "main" }}</span>
              </div>
              <div class="flex items-center justify-between border-b border-dashed pb-2">
                <span class="text-muted-foreground">Remote</span>
                <span class="font-mono text-[10px] truncate max-w-[250px]" :title="gitStatus.remoteUrl">
                  {{ gitStatus.remoteUrl || "No remote configured" }}
                </span>
              </div>
              <div class="flex items-center justify-between border-b border-dashed pb-2">
                <span class="text-muted-foreground">Stage</span>
                <span :class="[
                  'font-bold',
                  gitStatus.hasUncommittedChanges
                    ? 'text-orange-500'
                    : 'text-green-600',
                ]">
                  {{
                    gitStatus.hasUncommittedChanges
                      ? "Changes Detected"
                      : "Clean"
                  }}
                </span>
              </div>
            </div>

            <!-- Git Action Buttons -->
            <div class="grid grid-cols-3 gap-2 mt-2">
              <Button variant="outline" size="sm" class="h-8 gap-2" 
                :disabled="!gitStatus.hasUncommittedChanges"
                @click="handleGitCommit">
                <Plus class="h-3 w-3" /> Commit
              </Button>
              <Button variant="outline" size="sm" class="h-8 gap-2" 
                :disabled="!gitStatus.remoteUrl"
                @click="handlePullGit(tab.serviceId, tab.serviceData.directory)">
                <RefreshCw class="h-3 w-3" /> Pull
              </Button>
              <Button variant="outline" size="sm" class="h-8 gap-2" 
                :disabled="!gitStatus.remoteUrl"
                @click="handlePushGit(tab.serviceId, tab.serviceData.directory)">
                <Save class="h-3 w-3" /> Push
              </Button>
            </div>
          </div>
          <div v-else class="p-3 bg-muted/20 rounded border border-dashed text-center space-y-2">
            <p class="text-muted-foreground">
              This service directory is not a git repository. Initialize git
              for automatic change tracking.
            </p>
            <Button variant="outline" size="sm" class="h-7 gap-1.5" @click="
              initGit(tab.serviceId, tab.serviceData.directory)
              ">
              <GitBranch class="h-3 w-3" /> Initialize Git
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Environments & Variables Section -->
    <div class="bg-card rounded-lg border shadow-sm p-6 overflow-hidden">
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-2">
          <Globe class="h-4 w-4 text-primary" />
          <h3 class="font-semibold uppercase tracking-wider">
            Shared Environments & Variables
          </h3>
        </div>
        <Button variant="ghost" size="sm" class="h-8 gap-2 text-primary hover:bg-primary/5"
          @click="addVariableToAll(tab.serviceData.environments)">
          <Plus class="h-3.5 w-3.5" /> Add New Variable
        </Button>
      </div>

      <div class="relative border rounded-md overflow-hidden bg-background">
        <Table>
          <TableHeader>
            <TableRow class="hover:bg-transparent bg-muted/30">
              <TableHead class="w-[180px] font-bold">Variable Name</TableHead>
              <TableHead v-for="env in tab.serviceData.environments" :key="env.name" class="min-w-[150px] border-l">
                <div class="flex flex-col gap-1 items-start py-2">
                  <div class="flex items-center gap-1.5">
                    <span :class="[
                      'w-1.5 h-1.5 rounded-full',
                      env.isUnsafe
                        ? 'bg-destructive shadow-[0_0_8px_rgba(239,68,68,0.5)]'
                        : 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]',
                    ]"></span>
                    <span class="font-bold">{{ env.name }}</span>
                  </div>
                  <div class="flex items-center gap-1.5 mt-1">
                    <Switch v-if="env.isUnsafe" v-model="env.isUnsafe" class="bg-destructive shadow-[0_0_8px_rgba(239,68,68,0.5)]" />
                    <Switch v-else v-model="env.isUnsafe" class="bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]" />
                    <span class="uppercase tracking-tighter font-bold text-muted-foreground opacity-70">Unsafe</span>
                  </div>
                </div>
              </TableHead>
              <TableHead class="w-12 border-l"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="varName in getUniqueVariableNames(
              tab.serviceData.environments,
            )" :key="varName" class="group h-11 hover:bg-muted/10 transition-colors">
              <TableCell class="p-0 font-medium">
                <input :value="varName" @blur="
                  (e) =>
                    syncVariableName(
                      tab.serviceData.environments,
                      varName,
                      (e.target as HTMLInputElement).value,
                    )
                " class="w-full h-11 bg-transparent border-none px-3 focus:outline-none focus:ring-0" />
              </TableCell>
              <TableCell v-for="env in tab.serviceData.environments" :key="env.name"
                class="p-0 border-l group/cell align-top">
                <div class="flex items-center h-full w-full relative">
                  <!-- Secret Linked Mode -->
                  <div v-if="getVariable(env, varName)?.secretKey"
                    class="flex-1 h-11 flex items-center px-4 gap-2 bg-amber-500/10 text-amber-600 dark:text-amber-400 font-mono text-xs">
                    <Lock class="h-3 w-3" />
                    <span class="truncate">{{ getVariable(env, varName)?.secretKey }}</span>
                  </div>

                  <!-- Value Input Mode -->
                  <input v-else :value="getVariable(env, varName)?.value || ''" @input="
                    (e) =>
                      syncVariableValue(
                        env,
                        varName,
                        (e.target as HTMLInputElement).value,
                      )
                  " class="w-full h-11 bg-transparent border-none px-4 focus:outline-none focus:ring-0 group-hover/cell:bg-muted/20 transition-all font-mono text-xs"
                    :placeholder="`Value for ${env.name}`" />

                  <!-- Secret Linker Trigger -->
                  <Popover>
                    <PopoverTrigger as-child>
                      <Button variant="ghost" size="icon"
                        class="h-6 w-6 absolute right-1 opacity-0 group-hover/cell:opacity-100 transition-opacity"
                        :class="{ 'opacity-100 text-amber-500': getVariable(env, varName)?.secretKey }">
                        <Key class="h-3.5 w-3.5" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent class="w-64 p-2" align="end">
                      <div class="space-y-2">
                        <div class="flex items-center justify-between pb-2 border-b">
                          <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Link
                            Secret</span>
                          <span class="text-[10px] text-muted-foreground">{{ secretsStore.secrets.length }}
                            available</span>
                        </div>

                        <div class="max-h-[200px] overflow-y-auto space-y-1">
                          <button v-for="secret in secretsStore.secrets" :key="secret"
                            class="w-full flex items-center justify-between px-2 py-1.5 text-xs rounded-sm hover:bg-muted text-left"
                            :class="{ 'bg-primary/10 text-primary': getVariable(env, varName)?.secretKey === secret }"
                            @click="syncVariableSecret(env, varName, secret)">
                            <span class="truncate font-mono">{{ secret }}</span>
                            <Check v-if="getVariable(env, varName)?.secretKey === secret" class="h-3 w-3" />
                          </button>
                          <div v-if="secretsStore.secrets.length === 0"
                            class="text-xs text-muted-foreground p-2 text-center">
                            No secrets found
                          </div>
                        </div>

                        <div v-if="getVariable(env, varName)?.secretKey" class="pt-2 border-t">
                          <Button variant="ghost" size="sm"
                            class="w-full h-7 text-destructive hover:text-destructive hover:bg-destructive/10 gap-2"
                            @click="syncVariableSecret(env, varName, undefined)">
                            <Unlock class="h-3 w-3" /> Unlink Secret
                          </Button>
                        </div>
                      </div>
                    </PopoverContent>
                  </Popover>
                </div>
              </TableCell>
              <TableCell class="p-0 border-l text-center">
                <button @click="removeVariable(tab.serviceData.environments, varName)"
                  class="p-1 text-muted-foreground hover:text-destructive opacity-20 group-hover:opacity-100 transition-opacity">
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
      <div v-if="
        tab.serviceData.environments &&
        getUniqueVariableNames(tab.serviceData.environments).length === 0
      "
        class="flex flex-col items-center justify-center py-12 text-muted-foreground border border-t-0 rounded-b-md bg-muted/5">
        <AlertCircle class="h-8 w-8 opacity-20 mb-2" />
        <p class="">
          No variables defined for this
          {{ tab.serviceData.directory ? "service" : "collection" }}.
        </p>
        <button class="text-primary hover:underline mt-1" @click="addVariableToAll(tab.serviceData.environments)">
          Add your first variable
        </button>
      </div>
    </div>
  </div>
</template>
