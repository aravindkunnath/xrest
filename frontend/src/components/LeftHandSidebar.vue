<script setup lang="ts">
import { useRoute } from "vue-router";
import { useI18n } from "@/composables/useI18n";
import { RiStackLine, RiEarthLine, RiSettings4Line } from "@remixicon/vue";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";

const route = useRoute();
const { t, locale } = useI18n();

const navItems = [
    {
        path: "/services",
        icon: RiStackLine,
        tooltipKey: "sidebar.services",
    },
    {
        path: "/environments",
        icon: RiEarthLine,
        tooltipKey: "sidebar.environments",
    },
    {
        path: "/settings",
        icon: RiSettings4Line,
        tooltipKey: "sidebar.settings",
    },
];

const toggleLocale = () => {
    locale.value = locale.value === "en" ? "fr" : "en";
};
</script>

<template>
    <TooltipProvider>
        <div
            class="w-12 border-rborder-border flex flex-col justify-between items-center py-4 h-full select-none flex-shrink-0"
        >
            <!-- Upper Section -->
            <div class="flex flex-col gap-4 w-full items-center">
                <Tooltip v-for="item in navItems" :key="item.path">
                    <TooltipTrigger as-child>
                        <router-link
                            :to="item.path"
                            class="relative w-10 h-10 flex items-center justify-center rounded-md transition-colors group"
                            :class="[
                                route.path === item.path
                                    ? 'text-primary'
                                    : 'text-muted-foreground',
                            ]"
                        >
                            <!-- Active state left line -->
                            <span
                                v-if="route.path === item.path"
                                class="absolute left-0 w-[3px] h-6 bg-primary rounded-r"
                            />
                            <component :is="item.icon" class="w-5 h-5" />
                        </router-link>
                    </TooltipTrigger>
                    <TooltipContent
                        side="right"
                        class="z-50 border px-2.5 py-1 text-sm rounded shadow-md"
                    >
                        {{ t(item.tooltipKey) }}
                    </TooltipContent>
                </Tooltip>
            </div>

            <!-- Lower Section (Locale Switcher Button) -->
            <div class="w-full flex items-center justify-center">
                <button
                    @click="toggleLocale"
                    class="w-8 h-8 rounded text-xs font-bold border border-border transition-all uppercase"
                    title="Switch Language"
                >
                    {{ locale }}
                </button>
            </div>
        </div>
    </TooltipProvider>
</template>
