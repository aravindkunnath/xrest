<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick, onUnmounted } from "vue";
import { parseInterpolation } from "@/lib/placeholders";
import { useSecretsStore } from "@/stores/secrets";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { Lock } from "lucide-vue-next";

const props = defineProps<{
  modelValue: string;
  variables?: Record<string, string>;
  environmentName?: string;
  placeholder?: string;
  class?: string;
  id?: string;
}>();

const emit = defineEmits(["update:modelValue"]);

const inputRef = ref<HTMLInputElement | null>(null);
const highlightRef = ref<HTMLDivElement | null>(null);
const textRef = ref<HTMLDivElement | null>(null);
const hotspotRef = ref<HTMLDivElement | null>(null);

const secretsStore = useSecretsStore();
const tokens = computed(() =>
  parseInterpolation(
    props.modelValue || "",
    props.variables || {},
    secretsStore.secrets,
  ),
);

const internalValue = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

// --- Autocomplete Logic ---
const showAutocomplete = ref(false);
const autocompleteSearch = ref("");
const autocompleteIndex = ref(0);
const autocompleteTriggerPos = ref(0); // Index of the '{{'
const autocompleteCoords = ref({ x: 0, y: 0 });

const filteredVariables = computed(() => {
  const vars = Object.keys(props.variables || {});
  const secretVars = secretsStore.secrets.map((s) => `secret.${s}`);
  const allVars = [...vars, ...secretVars];

  if (!autocompleteSearch.value) return allVars;
  const query = autocompleteSearch.value.toLowerCase();
  return allVars.filter((v) => v.toLowerCase().includes(query));
});

const getCaretCoordinates = () => {
  if (!inputRef.value) return { x: 0, y: 0 };

  const selectionStart = inputRef.value.selectionStart || 0;
  const textBefore = props.modelValue.substring(0, selectionStart);

  // Create a mirror element to measure text width
  const mirror = document.createElement("div");
  const style = window.getComputedStyle(inputRef.value);

  mirror.style.position = "absolute";
  mirror.style.visibility = "hidden";
  mirror.style.whiteSpace = "pre";
  mirror.style.font = style.font;
  mirror.style.fontFamily = style.fontFamily;
  mirror.style.fontSize = style.fontSize;
  mirror.style.padding = style.padding;
  mirror.style.border = style.border;
  mirror.style.boxSizing = style.boxSizing;
  mirror.style.letterSpacing = style.letterSpacing;

  // Replace space with non-breaking space for accurate measurement if needed,
  // but since we use whitespace-pre and mono font, simple text is fine.
  mirror.textContent = textBefore;

  document.body.appendChild(mirror);
  const rect = inputRef.value.getBoundingClientRect();
  const textWidth = mirror.getBoundingClientRect().width;
  document.body.removeChild(mirror);

  // Calculate coordinates relative to viewport
  // We need to account for horizontal scroll of the input
  const x = rect.left + textWidth - inputRef.value.scrollLeft;
  const y = rect.bottom + 4; // Show below the input

  return { x, y };
};

const handleInput = (e: Event) => {
  syncScroll();
  const target = e.target as HTMLInputElement;
  const pos = target.selectionStart || 0;
  const text = target.value;

  // Check if we just typed '{{' or are inside one
  const textBefore = text.substring(0, pos);
  const lastOpening = textBefore.lastIndexOf("{{");
  const lastClosing = textBefore.lastIndexOf("}}");

  if (lastOpening !== -1 && lastOpening > lastClosing) {
    showAutocomplete.value = true;
    autocompleteTriggerPos.value = lastOpening;
    autocompleteSearch.value = textBefore.substring(lastOpening + 2);
    autocompleteIndex.value = 0;
    nextTick(() => {
      autocompleteCoords.value = getCaretCoordinates();
    });
  } else {
    showAutocomplete.value = false;
  }
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!showAutocomplete.value) return;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    autocompleteIndex.value =
      (autocompleteIndex.value + 1) % filteredVariables.value.length;
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    autocompleteIndex.value =
      (autocompleteIndex.value - 1 + filteredVariables.value.length) %
      filteredVariables.value.length;
  } else if (e.key === "Enter" || e.key === "Tab") {
    if (filteredVariables.value.length > 0) {
      e.preventDefault();
      selectVariable(filteredVariables.value[autocompleteIndex.value]);
    }
  } else if (e.key === "Escape") {
    e.preventDefault();
    showAutocomplete.value = false;
  }
};

const selectVariable = (varName: string) => {
  const text = props.modelValue;
  const before = text.substring(0, autocompleteTriggerPos.value);
  const after = text.substring(inputRef.value?.selectionStart || 0);

  const newValue = `${before}{{${varName}}}${after}`;
  emit("update:modelValue", newValue);
  showAutocomplete.value = false;

  nextTick(() => {
    if (inputRef.value) {
      const newPos = before.length + varName.length + 4; // 4 for {{ and }}
      inputRef.value.setSelectionRange(newPos, newPos);
      inputRef.value.focus();
    }
  });
};

const syncScroll = () => {
  if (inputRef.value) {
    const scrollLeft = inputRef.value.scrollLeft;
    if (highlightRef.value) highlightRef.value.scrollLeft = scrollLeft;
    if (textRef.value) textRef.value.scrollLeft = scrollLeft;
    if (hotspotRef.value) hotspotRef.value.scrollLeft = scrollLeft;
  }
};

const handleHotspotClick = () => {
  inputRef.value?.focus();
};

const dropdownRef = ref<HTMLElement | null>(null);

const handleClickOutside = (e: MouseEvent) => {
  if (!showAutocomplete.value) return;

  const target = e.target as HTMLElement;
  const isInput = inputRef.value?.contains(target);
  const isDropdown = dropdownRef.value?.contains(target);

  if (!isInput && !isDropdown) {
    showAutocomplete.value = false;
  }
};

const handleBlur = () => {
  // Delay closing to allow clicks on the dropdown to register
  setTimeout(() => {
    showAutocomplete.value = false;
  }, 200);
};

onMounted(() => {
  syncScroll();
  window.addEventListener(
    "scroll",
    () => {
      if (showAutocomplete.value) showAutocomplete.value = false;
    },
    true,
  );
  window.addEventListener("mousedown", handleClickOutside);
});

onUnmounted(() => {
  window.removeEventListener(
    "scroll",
    () => {
      if (showAutocomplete.value) showAutocomplete.value = false;
    },
    true,
  );
  window.removeEventListener("mousedown", handleClickOutside);
});

watch(
  () => props.modelValue,
  () => {
    nextTick(syncScroll);
  },
);
</script>

<template>
  <div
    class="relative w-full group overflow-hidden rounded-md border bg-background transition-shadow focus-within:ring-1 focus-within:ring-primary">
    <!-- 1. Base Text Layer (Visible) -->
    <div ref="textRef"
      class="absolute inset-0 pointer-events-none px-3 py-1.5 font-mono whitespace-pre overflow-hidden flex items-center h-full leading-none"
      :style="{ fontSize: 'inherit' }">
      <template v-for="(token, i) in tokens" :key="i">
        <span v-if="token.type === 'text'" class="text-foreground">{{
          token.content
          }}</span>
        <span v-else class="text-transparent selection:text-transparent">{{
          token.content
          }}</span>
      </template>
    </div>

    <!-- 2. Highlight Layer (Colored backgrounds/text for variables) -->
    <div ref="highlightRef"
      class="absolute inset-0 pointer-events-none px-3 py-1.5 font-mono whitespace-pre overflow-hidden flex items-center h-full leading-none"
      :style="{ fontSize: 'inherit' }">
      <template v-for="(token, i) in tokens" :key="i">
        <span v-if="token.type === 'text'" class="text-transparent">{{
          token.content
          }}</span>
        <span v-else :class="cn(
          'rounded transition-all leading-none h-[calc(100%-4px)] flex items-center px-0.5',
          token.isSecret
            ? 'text-amber-500 bg-amber-500/20'
            : token.isValid
              ? 'text-primary bg-primary/20'
              : 'text-destructive bg-destructive/20 underline decoration-destructive/50 underline-offset-2',
        )
          ">
          {{ token.content }}
        </span>
      </template>
    </div>

    <!-- 3. Real Input (Transparent text, visible caret) -->
    <input :id="id" ref="inputRef" v-model="internalValue" type="text" :placeholder="placeholder" :class="cn(
      'w-full bg-transparent border-none py-1.5 px-3 focus:outline-none font-mono h-full relative z-10 leading-none',
      'text-transparent caret-foreground selection:bg-primary/30',
      props.class,
    )
      " :style="{ fontSize: 'inherit' }" @scroll="syncScroll" @input="handleInput" @keydown="handleKeyDown"
      @blur="handleBlur" spellcheck="false" autocapitalize="off" autocorrect="off" />

    <!-- 4. Hotspot Layer (Invisible tokens for tooltips on TOP of everything) -->
    <div ref="hotspotRef"
      class="absolute inset-0 pointer-events-none px-3 py-1.5 font-mono whitespace-pre overflow-hidden flex items-center h-full z-20 leading-none"
      :style="{ fontSize: 'inherit' }">
      <template v-for="(token, i) in tokens" :key="i">
        <span v-if="token.type === 'text'" class="text-transparent">{{
          token.content
          }}</span>
        <template v-else>
          <TooltipProvider :delay-duration="0">
            <Tooltip>
              <TooltipTrigger as-child>
                <span class="rounded text-transparent pointer-events-auto cursor-help"
                  @mousedown.prevent="handleHotspotClick">
                  {{ token.content }}
                </span>
              </TooltipTrigger>
              <TooltipContent side="top" class="w-fit max-w-[280px] break-all z-[100] shadow-xl border-primary/20 px-3 py-2">
                <div class="space-y-1.5 p-0.5">
                  <div v-if="token.isValid" class="flex flex-col gap-1">
                    <div class="flex items-center justify-between gap-4">
                      <div class="flex items-center gap-2">
                        <div :class="cn(
                          'h-1.5 w-1.5 rounded-full animate-pulse',
                          token.isSecret ? 'bg-amber-500' : 'bg-primary',
                        )
                          "></div>
                        <span class="font-semibold text-muted-foreground text-xs">{{
                          token.isSecret ? "Secure Secret" : "Active Value"
                        }}</span>
                      </div>
                      <span v-if="!token.isSecret && environmentName"
                        class="bg-muted px-1 py-0.5 rounded text-muted-foreground font-medium text-[10px] border border-border/50">{{
                        environmentName }}</span>
                      <Lock v-if="token.isSecret" class="h-3 w-3 text-amber-500" />
                    </div>
                    <div v-if="token.isSecret"
                      class="text-amber-500 bg-amber-500/5 p-1.5 rounded border border-amber-500/10 text-xs italic">
                      Value is securely stored and hidden.
                    </div>
                    <span v-else
                      class="text-primary bg-primary/5 p-1 rounded border border-primary/10 break-all select-all font-mono text-xs">{{
                      token.resolvedValue }}</span>
                  </div>
                  <div v-else class="flex flex-col gap-1 text-destructive">
                    <div class="flex items-center justify-between gap-4">
                      <div class="flex items-center gap-2">
                        <div class="h-1.5 w-1.5 rounded-full bg-destructive animate-bounce"></div>
                        <span class="font-semibold text-xs">Missing Variable</span>
                      </div>
                      <span v-if="environmentName"
                        class="bg-destructive/10 px-1 py-0.5 rounded text-destructive font-medium text-[10px] border border-destructive/20">{{
                        environmentName }}</span>
                    </div>
                    <span class="leading-relaxed text-xs">The variable
                      <code class="font-bold underline underline-offset-2">{{ token.content }}</code> is not defined.</span>
                  </div>
                </div>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </template>
      </template>
    </div>

    <!-- Autocomplete Dropdown -->
    <Teleport to="body">
      <div v-if="showAutocomplete && filteredVariables.length > 0" ref="dropdownRef"
        class="fixed z-[9999] bg-popover text-popover-foreground border shadow-xl rounded-md overflow-hidden min-w-[200px] animate-in fade-in zoom-in-95 duration-100"
        :style="{
          top: autocompleteCoords.y + 'px',
          left: autocompleteCoords.x + 'px',
        }">
        <div class="p-1 max-h-[300px] overflow-auto">
          <div v-for="(varName, idx) in filteredVariables" :key="varName" @mouseenter="autocompleteIndex = idx"
            @mousedown.prevent="selectVariable(varName)" :class="cn(
              'flex items-center justify-between px-2 py-1.5 rounded-sm cursor-pointer transition-colors',
              idx === autocompleteIndex
                ? 'bg-accent text-accent-foreground'
                : 'hover:bg-accent/50',
            )
              ">
            <div class="flex items-center gap-2">
              <div :class="cn(
                'h-2 w-2 rounded-full',
                varName.startsWith('secret.')
                  ? 'bg-amber-500'
                  : 'bg-primary/40',
              )
                "></div>
              <span class="font-bold">{{ varName }}</span>
            </div>
            <span class="text-muted-foreground opacity-50 truncate max-w-[100px]">
              {{
                varName.startsWith("secret.")
                  ? "••••••••"
                  : variables?.[varName]
              }}
            </span>
          </div>
        </div>
        <div class="border-t bg-muted/30 px-2 py-1 flex items-center justify-between">
          <span class="text-muted-foreground uppercase font-bold tracking-widest text-[10px]">Variables & Secrets</span>
          <span class="text-muted-foreground opacity-50 text-[10px]">{{ filteredVariables.length }} found</span>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
input,
div {
  font-family:
    "Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco,
    Consolas, "Liberation Mono", "Courier New", monospace;
  font-size: 12pt;
  letter-spacing: normal;
  line-height: normal;
}

div::-webkit-scrollbar {
  display: none;
}

div {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
