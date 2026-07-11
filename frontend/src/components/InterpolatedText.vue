<script setup lang="ts">
import { computed } from "vue";
import { parseInterpolation } from "@/lib/placeholders";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const props = defineProps<{
  text: string;
  variables?: Record<string, string>;
  environmentName?: string;
  class?: string;
}>();

const tokens = computed(() =>
  parseInterpolation(props.text || "", props.variables || {}),
);
</script>

<template>
  <span
    :class="
      cn('inline-flex flex-wrap items-center gap-0 font-mono', props.class)
    "
  >
    <template v-for="(token, i) in tokens" :key="i">
      <span
        v-if="token.type === 'text'"
        class="whitespace-pre-wrap break-all"
        >{{ token.content }}</span
      >
      <template v-else>
        <TooltipProvider :delay-duration="0">
          <Tooltip>
            <TooltipTrigger as-child>
              <span
                :class="
                  cn(
                    'px-0.5 rounded font-bold cursor-default transition-all mx-px whitespace-pre',
                    token.isValid
                      ? 'text-primary bg-primary/10'
                      : 'text-destructive bg-destructive/10 underline decoration-destructive/30',
                  )
                "
              >
                {{ token.content }}
              </span>
            </TooltipTrigger>
            <TooltipContent side="top" class="max-w-xs break-all z-[100]">
              <div class="space-y-1">
                <div v-if="token.isValid" class="flex flex-col gap-1">
                  <div class="flex items-center justify-between gap-4">
                    <span class="uppercase font-bold text-muted-foreground"
                      >Current Value</span
                    >
                    <span
                      v-if="environmentName"
                      class="bg-muted px-1.5 py-0.5 rounded text-muted-foreground font-bold border border-border/50"
                      >{{ environmentName }}</span
                    >
                  </div>
                  <span class="text-primary">{{ token.resolvedValue }}</span>
                </div>
                <div v-else class="flex flex-col gap-1">
                  <div class="flex items-center justify-between gap-4">
                    <span class="font-bold text-destructive"
                      >Unresolved Variable</span
                    >
                    <span
                      v-if="environmentName"
                      class="bg-destructive/10 px-1.5 py-0.5 rounded text-destructive font-bold border border-destructive/20"
                      >{{ environmentName }}</span
                    >
                  </div>
                </div>
              </div>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </template>
    </template>
  </span>
</template>
