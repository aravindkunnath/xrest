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
            <Label for="theme-select" class="font-medium">Theme</Label>
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
                    <span class="w-2 h-2 rounded-full bg-white border"></span>
                    Light
                  </div>
                </SelectItem>
                <SelectItem value="dark">
                  <div class="flex items-center gap-2">
                    <span class="w-2 h-2 rounded-full bg-black"></span>
                    Dark
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
            <p class="text-muted-foreground">
              Select between light and dark mode, or follow your system
              preference.
            </p>
          </div>
        </div>
      </section>

      <Separator />

      <!-- More sections can be added here -->
    </div>
  </div>
</template>
