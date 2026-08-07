<script setup lang="ts">
import { useI18n } from "@/composables/useI18n";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Button } from "@/components/ui/button";
import { RiGlobalLine, RiCheckLine } from "@remixicon/vue";
import { useSettingsStore } from "@/stores/settings";
import { Columns2, Rows2, History } from "@lucide/vue";
import { useDialogState } from "@/composables/useDialogState";

const { locale, availableLocales } = useI18n();
const settingsStore = useSettingsStore();
const { openHistoryOverlay } = useDialogState();

const setLocale = (lang: string) => {
    locale.value = lang;
};
</script>

<template>
    <footer class="h-9 w-full border-t border-border bg-sidebar px-4 flex items-center justify-between text-xs text-muted-foreground select-none shrink-0 z-40">
        <!-- Left Side: Status / Info -->
        <div class="flex items-center gap-2">
        </div>

        <!-- Right Side: Locale Dropup & More Options -->
        <div class="flex items-center gap-2">
            <Button
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs font-medium gap-1.5 hover:bg-accent hover:text-accent-foreground cursor-pointer"
                @click="openHistoryOverlay"
                title="View Request History"
            >
                <History class="w-3.5 h-3.5" />
                <span>History</span>
            </Button>
            <!-- Layout Switcher -->
            <div class="flex items-center border-r border-border pr-2 mr-1 gap-1">
                <Button
                    variant="ghost"
                    size="sm"
                    class="h-7 w-7 p-0 hover:bg-accent hover:text-accent-foreground cursor-pointer"
                    :class="{ 'bg-accent text-accent-foreground': settingsStore.layout === 'horizontal' }"
                    @click="settingsStore.layout = 'horizontal'"
                    title="Horizontal Split (Top/Bottom)"
                >
                    <Rows2 class="w-3.5 h-3.5" />
                </Button>
                <Button
                    variant="ghost"
                    size="sm"
                    class="h-7 w-7 p-0 hover:bg-accent hover:text-accent-foreground cursor-pointer"
                    :class="{ 'bg-accent text-accent-foreground': settingsStore.layout === 'vertical' }"
                    @click="settingsStore.layout = 'vertical'"
                    title="Vertical Split (Left/Right)"
                >
                    <Columns2 class="w-3.5 h-3.5" />
                </Button>
            </div>
            <Popover>
                <PopoverTrigger as-child>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-7 px-2.5 text-xs font-medium gap-1.5 uppercase hover:bg-accent hover:text-accent-foreground cursor-pointer"
                    >
                        <RiGlobalLine class="w-3.5 h-3.5" />
                        <span>{{ locale }}</span>
                    </Button>
                </PopoverTrigger>
                <PopoverContent
                    side="top"
                    align="end"
                    class="w-48 p-1 z-50 bg-popover border border-border shadow-lg rounded-md animate-in fade-in-50 slide-in-from-bottom-1"
                >
                    <div class="px-2 py-1.5 text-[10px] font-semibold text-muted-foreground tracking-wider uppercase">
                        Language
                    </div>
                    <div class="flex flex-col gap-0.5">
                        <button
                            v-for="lang in availableLocales"
                            :key="lang"
                            @click="setLocale(lang)"
                            class="flex items-center justify-between w-full px-2 py-1.5 text-xs rounded-sm hover:bg-accent hover:text-accent-foreground text-left cursor-pointer transition-colors"
                            :class="{ 'font-semibold text-primary': locale === lang }"
                        >
                            <span>{{ lang === 'en' ? 'English' : 'Français' }}</span>
                            <RiCheckLine v-if="locale === lang" class="w-3.5 h-3.5 text-primary" />
                        </button>
                    </div>
                    <div class="border-t border-border my-1"></div>
                    <div class="px-2 py-1 text-[10px] text-muted-foreground/60 italic">
                        More languages coming soon
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    </footer>
</template>
