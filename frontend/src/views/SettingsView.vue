<script setup lang="ts">
import { useSettingsStore } from "@/stores/settings";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";

const settingsStore = useSettingsStore();

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
</script>

<template>
    <div class="p-8 max-w-4xl">
        <div class="mb-8">
            <h1 class="text-3xl font-bold tracking-tight">Settings</h1>
            <p class="text-muted-foreground mt-2">
                Manage your interface preferences and application configuration.
            </p>
        </div>

        <Separator class="my-8" />

        <div class="space-y-12">
            <!-- Appearance Section -->
            <section class="grid grid-cols-1 md:grid-cols-3 gap-8">
                <div>
                    <h2 class="text-lg font-semibold">Appearance</h2>
                    <p class="text-muted-foreground mt-1">
                        Customize how xrest looks on your screen.
                    </p>
                </div>

                <div class="md:col-span-2 space-y-6">
                    <div class="space-y-4">
                        <Label for="theme-select" class="font-medium"
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
                        <p class="text-muted-foreground">
                            Select between light and dark mode, or follow your
                            system preference.
                        </p>
                    </div>

                    <Separator />

                    <div class="space-y-4">
                        <div
                            class="flex items-center justify-between w-[320px]"
                        >
                            <Label class="font-medium"
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

                        <p class="text-muted-foreground">
                            Current Zoom:
                            <span class="font-mono font-medium"
                                >{{ 100 + settingsStore.zoomLevel * 10 }}%</span
                            >
                        </p>
                    </div>
                </div>
            </section>

            <Separator />

            <!-- More sections can be added here -->
        </div>
    </div>
</template>
