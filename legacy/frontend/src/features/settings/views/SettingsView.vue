<script setup lang="ts">
import { computed, ref } from "vue";
import { useSettingsStore, MIN_VERSIONS_LIMIT, MAX_VERSIONS_LIMIT, VERSIONS_LIMIT_STEP } from "@/stores/settings";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import SecretsView from "@/features/secrets/views/SecretsView.vue";
import { Settings2, Shield } from "@lucide/vue";

const settingsStore = useSettingsStore();

const props = withDefaults(
    defineProps<{
        initialSection?: string;
    }>(),
    {
        initialSection: "general",
    }
);

const activeSection = ref(props.initialSection);

const navItems = [
    { id: "general", label: "General", icon: Settings2 },
    { id: "secrets", label: "Secrets", icon: Shield },
];

const setZoom = (level: number) => {
    if (level >= -2 && level <= 5) {
        settingsStore.setZoomLevel(level);
    }
};

const resetZoom = () => {
    settingsStore.setZoomLevel(0);
};

// Define scale steps for Apple-like display rendering
const zoomSteps = [-2, -1, 0, 1, 2, 3, 4, 5];

// Version history retention: 5..50 in increments of 5.
const versionsLimitOptions = Array.from(
    { length: (MAX_VERSIONS_LIMIT - MIN_VERSIONS_LIMIT) / VERSIONS_LIMIT_STEP + 1 },
    (_, i) => MIN_VERSIONS_LIMIT + i * VERSIONS_LIMIT_STEP,
);

// reka-ui selects match values as strings; drive them with the numeric setting.
const versionsLimitModel = computed({
    get: () => String(settingsStore.versionsLimit),
    set: (value: any) => settingsStore.setVersionsLimit(Number(value)),
});
</script>

<template>
    <div class="h-full flex overflow-hidden bg-background">
        <!-- Settings Navigation Sidebar -->
        <aside class="w-60 border-r bg-muted/10 p-4 shrink-0 flex flex-col select-none">
            <div class="space-y-4">
                <div class="px-3 py-2">
                    <h2 class="text-base font-semibold tracking-tight">App Settings</h2>
                    <p class="text-xs text-muted-foreground mt-0.5">Preferences & Security</p>
                </div>
                <nav class="space-y-1">
                    <button
                        v-for="item in navItems"
                        :key="item.id"
                        @click="activeSection = item.id"
                        :class="[
                            'w-full flex items-center gap-2.5 px-3 py-2 text-xs font-medium rounded-md transition-colors cursor-pointer text-left',
                            activeSection === item.id
                                ? 'bg-primary text-primary-foreground shadow-xs font-semibold'
                                : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
                        ]"
                    >
                        <component :is="item.icon" class="h-4 w-4 shrink-0" />
                        <span>{{ item.label }}</span>
                    </button>
                </nav>
            </div>
        </aside>

        <!-- Settings Main Content Pane -->
        <main class="flex-1 overflow-auto p-8 w-full">
            <!-- General / Appearance Section -->
            <div v-if="activeSection === 'general'" class="space-y-8 animate-in fade-in-50 duration-150 w-full">
                <div>
                    <h1 class="text-2xl font-bold tracking-tight">General Settings</h1>
                    <p class="text-muted-foreground text-sm mt-1">
                        Manage your interface preferences and application display.
                    </p>
                </div>

                <Separator />

                <div class="space-y-12">
                    <!-- Appearance Section -->
                    <section class="grid grid-cols-1 md:grid-cols-3 gap-8">
                        <div>
                            <h2 class="text-base font-semibold">Appearance</h2>
                            <p class="text-muted-foreground text-xs mt-1">
                                Customize how xrest looks on your screen.
                            </p>
                        </div>

                        <div class="md:col-span-2 space-y-6">
                            <div class="space-y-3">
                                <Label for="theme-select" class="text-xs font-medium"
                                    >Theme</Label
                                >
                                <Select v-model="settingsStore.mode">
                                    <SelectTrigger id="theme-select" class="w-[240px]">
                                        <SelectValue placeholder="Select theme" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="auto">
                                            <div class="flex items-center gap-2">
                                                <span
                                                    class="w-2 h-2 rounded-full bg-linear-to-r from-white to-black border"
                                                ></span>
                                                System
                                            </div>
                                        </SelectItem>
                                        <SelectItem value="light">
                                            <div class="flex items-center gap-2">
                                                <span
                                                    class="w-2 h-2 rounded-full bg-white border"
                                                ></span>
                                                Light
                                            </div>
                                        </SelectItem>
                                        <SelectItem value="dark">
                                            <div class="flex items-center gap-2">
                                                <span
                                                    class="w-2 h-2 rounded-full bg-black"
                                                ></span>
                                                Dark
                                            </div>
                                        </SelectItem>
                                    </SelectContent>
                                </Select>
                                <p class="text-xs text-muted-foreground">
                                    Select between light and dark mode, or follow your
                                    system preference.
                                </p>
                            </div>

                            <Separator />

                            <div class="space-y-3">
                                <div
                                    class="flex items-center justify-between w-[320px]"
                                >
                                    <Label class="text-xs font-medium"
                                        >Application Font Size</Label
                                    >
                                    <button
                                        v-if="settingsStore.zoomLevel !== 0"
                                        @click="resetZoom"
                                        class="text-xs text-primary hover:underline font-medium focus:outline-none cursor-pointer"
                                    >
                                        Reset to 100%
                                    </button>
                                </div>

                                <!-- Apple-style Segmented Display Zoom Track -->
                                <div
                                    class="flex items-center gap-3 w-[320px] select-none"
                                    style="font-size: 14px !important"
                                >
                                    <!-- Smaller 'A' indicator -->
                                    <span
                                        class="text-xs font-semibold text-muted-foreground"
                                        >A</span
                                    >

                                    <!-- Track Segment container -->
                                    <div
                                        class="flex-1 flex items-center bg-muted/30 border rounded-full h-8 p-1 relative overflow-hidden"
                                    >
                                        <button
                                            v-for="step in zoomSteps"
                                            :key="step"
                                            @click="setZoom(step)"
                                            class="flex-1 h-full rounded-full transition-all duration-150 relative z-10 focus:outline-none cursor-pointer"
                                            :class="[
                                                settingsStore.zoomLevel === step
                                                    ? 'bg-accent text-accent-foreground shadow-sm font-semibold border border-border/80'
                                                    : 'text-muted-foreground/60 hover:text-foreground hover:bg-muted/10',
                                            ]"
                                        >
                                            <span
                                                class="absolute inset-0 flex items-center justify-center text-[10px]"
                                            >
                                                {{ step === 0 ? "•" : "" }}
                                            </span>
                                        </button>
                                    </div>

                                    <!-- Larger 'A' indicator -->
                                    <span
                                        class="text-lg font-semibold text-muted-foreground"
                                        >A</span
                                    >
                                </div>

                                <p class="text-xs text-muted-foreground">
                                    Current Zoom:
                                    <span class="font-mono font-medium"
                                        >{{ 100 + settingsStore.zoomLevel * 10 }}%</span
                                    >
                                </p>
                            </div>
                        </div>
                    </section>

                    <!-- Version History Section -->
                    <section class="grid grid-cols-1 md:grid-cols-3 gap-8">
                        <div>
                            <h2 class="text-base font-semibold">Version History</h2>
                            <p class="text-muted-foreground text-xs mt-1">
                                Control how many request versions are kept per
                                endpoint.
                            </p>
                        </div>

                        <div class="md:col-span-2 space-y-6">
                            <div class="space-y-3">
                                <Label for="versions-limit-select" class="text-xs font-medium"
                                    >Versions Kept per Endpoint</Label
                                >
                                <Select v-model="versionsLimitModel">
                                    <SelectTrigger id="versions-limit-select" class="w-[240px]">
                                        <SelectValue placeholder="Select version limit" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem
                                            v-for="opt in versionsLimitOptions"
                                            :key="opt"
                                            :value="String(opt)"
                                        >
                                            {{ opt }} versions
                                        </SelectItem>
                                    </SelectContent>
                                </Select>
                                <p class="text-xs text-muted-foreground">
                                    When a new version is added, the oldest
                                    versions beyond this limit are removed
                                    (first-in, first-out).
                                </p>
                            </div>
                        </div>
                    </section>
                </div>
            </div>

            <!-- Secrets Section -->
            <div v-else-if="activeSection === 'secrets'" class="space-y-6 animate-in fade-in-50 duration-150 w-full">
                <div>
                    <h1 class="text-2xl font-bold tracking-tight">Workspace Secrets</h1>
                    <p class="text-muted-foreground text-sm mt-1">
                        Securely store keys, tokens, and credentials in the system keyring.
                    </p>
                </div>

                <Separator />

                <SecretsView :embedded="true" />
            </div>
        </main>
    </div>
</template>
