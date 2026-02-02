<script setup lang="ts">
import { AlertCircle } from "lucide-vue-next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

defineProps<{
  open: boolean;
  environmentName: string;
  countdown: number;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "proceed"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent
      class="sm:max-w-[425px] border-destructive shadow-lg shadow-destructive/20 transition-all duration-300"
    >
      <DialogHeader>
        <div class="flex items-center gap-3 mb-2">
          <div
            class="h-10 w-10 rounded-full bg-destructive/10 flex items-center justify-center"
          >
            <AlertCircle class="h-6 w-6 text-destructive animate-pulse" />
          </div>
          <DialogTitle class="text-xl font-bold text-destructive"
            >Unsafe Environment!</DialogTitle
          >
        </div>
        <DialogDescription
          class="text-foreground/80 leading-relaxed font-medium"
        >
          You are about to execute a request against
          <span
            class="px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 font-bold border border-amber-200 dark:border-amber-700 mx-1"
            >{{ environmentName }}</span
          >. This environment is marked as unsafe.
        </DialogDescription>
      </DialogHeader>

      <div class="py-6 flex flex-col items-center gap-4">
        <p class="text-muted-foreground text-center">
          Are you sure you want to proceed with this action?
        </p>
        <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
          <div
            class="h-full bg-destructive transition-all duration-1000 ease-linear"
            :style="{ width: `${(countdown / 10) * 100}%` }"
          ></div>
        </div>
        <p
          class="uppercase font-bold tracking-widest text-muted-foreground/60 italic"
        >
          Request will be cancelled in
          <span class="text-destructive font-bold tabular-nums"
            >{{ countdown }}s</span
          >
        </p>
      </div>

      <DialogFooter class="sm:justify-start gap-2">
        <Button
          variant="destructive"
          class="flex-1 h-11 font-bold shadow-sm shadow-destructive/20 border-b-4 border-destructive-foreground/10 active:border-b-0 active:translate-y-1 transition-all"
          @click="emit('proceed')"
        >
          I Accept the Risk
        </Button>
        <Button
          variant="outline"
          class="px-8 h-11 font-medium"
          @click="emit('cancel')"
        >
          Cancel
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
