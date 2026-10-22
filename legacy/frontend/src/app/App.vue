<script setup lang="ts">
import TitleBar from "@/components/common/TitleBar.vue";
import MainLayout from "@/app/layouts/MainLayout.vue";
import { useSettingsStore } from "@/stores/settings";
import HistoryOverlay from "@/features/history/components/HistoryOverlay.vue";
import { onMounted } from "vue";

const settingsStore = useSettingsStore();

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
        <TitleBar />
        <MainLayout />
        <HistoryOverlay />
    </div>
</template>

<style scoped>
.app-container {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
    padding-top: 40px;
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
