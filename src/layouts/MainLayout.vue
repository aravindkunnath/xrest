<script setup lang="ts">
import { SidebarInset } from "@/components/ui/sidebar";
import "vue-sonner/style.css";
import { Toaster } from "@/components/ui/sonner";
</script>

<template>
  <SidebarInset class="flex flex-col overflow-hidden" style="height: 100vh; padding-top: 40px;">
    <main class="flex-1 overflow-hidden">
      <router-view />
    </main>
  </SidebarInset>
  <Toaster />
</template>
