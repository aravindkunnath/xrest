<script setup lang="ts">
import { SidebarInset } from "@/components/ui/sidebar";
import "vue-sonner/style.css";
import { Toaster } from "@/components/ui/sonner";
import VSCodeSidebar from "@/components/VSCodeSidebar.vue";
</script>

<template>
  <div class="flex h-full w-full overflow-hidden">
    <VSCodeSidebar />
    <SidebarInset class="bg-transparent flex flex-col flex-1" style="background-color: transparent !important;">
      <router-view />
    </SidebarInset>
    <Toaster />
  </div>
</template>
