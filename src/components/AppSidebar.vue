<script setup lang="ts">
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { History, Layers, LayoutGrid, Settings, Key } from "lucide-vue-next";
import { useRoute } from "vue-router";

const route = useRoute();

const navItems = [
  {
    title: "Services",
    url: "/services",
    icon: Layers,
  },
  {
    title: "Collections",
    url: "/collections",
    icon: LayoutGrid,
  },
  {
    title: "Secrets",
    url: "/secrets",
    icon: Key,
  },
  {
    title: "History",
    url: "/history",
    icon: History,
  },
  {
    title: "Settings",
    url: "/settings",
    icon: Settings,
  },
];
</script>

<template>
  <Sidebar side="right" collapsible="icon">
    <SidebarHeader class="m-8">

    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in navItems" :key="item.title">
              <SidebarMenuButton as-child :tooltip="item.title" :is-active="route.path === item.url">
                <router-link :to="item.url" class="flex items-center gap-2">
                  <component :is="item.icon" class="h-6 w-6" />
                  <span>{{ item.title }}</span>
                </router-link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarRail />
  </Sidebar>
</template>
