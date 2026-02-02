<script setup lang="ts">
import MainLayout from "@/layouts/MainLayout.vue";
import { useSettingsStore } from "@/stores/settings";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { History, Key, Layers, Settings } from "lucide-vue-next";
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

const handleDrag = async (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (target.closest('button') || target.closest('a') || target.closest('input')) {
    return;
  }

  if (event.button === 0 && event.detail === 1) {
    await getCurrentWindow().startDragging();
  }
};

const handleDoubleClick = async () => {
  const win = getCurrentWindow();
  if (await win.isMaximized()) {
    await win.unmaximize();
  } else {
    await win.maximize();
  }
};

onMounted(async () => {
  try {
    await settingsStore.loadSettings();
  } catch (e) {
    console.error("Failed to initialize settings:", e);
  } finally {
    await getCurrentWindow().show();
  }
});
</script>

<template>
  <div class="app-container">
    <div class="titlebar" data-tauri-drag-region @mousedown="handleDrag" @dblclick="handleDoubleClick">
      <div class="flex items-center justify-center flex-1 ml-[72px] pr-[72px]">
        <nav
          class="flex items-center gap-1 bg-muted/30 p-1 rounded-lg backdrop-blur-md border border-white/5 pointer-events-auto">
          <router-link v-for="item in navItems" :key="item.title" :to="item.url"
            class="flex items-center gap-2 px-3 py-1 rounded-md transition-all duration-200 text-sm font-medium hover:bg-white/10"
            :class="route.path === item.url ? 'bg-primary/20 text-primary' : 'text-muted-foreground'">
            <component :is="item.icon" class="h-4 w-4" />
            <span>{{ item.title }}</span>
          </router-link>
        </nav>
      </div>
    </div>
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
}

.titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 40px;
  z-index: 9999;
  cursor: default;
  -webkit-user-select: none;
  user-select: none;
  display: flex;
  align-items: center;
  padding: 0 4px;
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
