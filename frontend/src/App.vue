<script setup lang="ts">
import TitleBar from "@/components/TitleBar.vue";
import MainLayout from "@/layouts/MainLayout.vue";
import { useSettingsStore } from "@/stores/settings";
import { History, Key, Layers, Settings } from "@lucide/vue";
import { onMounted } from "vue";
import { useRoute } from "vue-router";

const settingsStore = useSettingsStore();
const route = useRoute();

const navItems = [
    { title: "Services", url: "/services", icon: Layers },
    // { title: "Collections", url: "/collections", icon: LayoutGrid },
    { title: "Secrets", url: "/secrets", icon: Key },
    { title: "History", url: "/history", icon: History },
    { title: "Settings", url: "/settings", icon: Settings },
];

onMounted(async () => {
    try {
        await settingsStore.loadSettings();
    } catch (e) {
        console.error("Failed to initialize settings:", e);
    }
});
</script>

<template>
    <div class="app-container">
        <TitleBar>
            <div class="flex items-center justify-center flex-1">
                <nav
                    class="flex items-center gap-1 bg-muted/80 p-1 rounded-lg border border-white/5 pointer-events-auto"
                >
                    <router-link
                        v-for="item in navItems"
                        :key="item.title"
                        :to="item.url"
                        class="flex items-center gap-2 px-3 py-1 rounded-md transition-all duration-200 text-sm font-medium hover:bg-white/10"
                        :class="
                            route.path === item.url
                                ? 'bg-primary/20 text-primary'
                                : 'text-muted-foreground'
                        "
                    >
                        <component :is="item.icon" class="h-4 w-4" />
                        <span>{{ item.title }}</span>
                    </router-link>
                </nav>
            </div>
        </TitleBar>
        <MainLayout />
    </div>
</template>

<style scoped>
.app-container {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
    padding-top: 30px;
}
</style>

<style>
/* Global styles if needed */
body {
    margin: 0;
    padding: 0;
    overflow: hidden;
}
</style>
