<script setup lang="ts">
import { SidebarInset } from "@/components/ui/sidebar";
import "vue-sonner/style.css";
import { Toaster } from "@/components/ui/sonner";
import LeftHandSidebar from "@/components/LeftHandSidebar.vue";
</script>

<template>
    <div class="flex h-full w-full overflow-hidden">
        <LeftHandSidebar />
        <SidebarInset
            class="bg-transparent flex flex-col flex-1"
            style="background-color: transparent !important"
        >
            <router-view />
        </SidebarInset>
        <Toaster />
    </div>
</template>
