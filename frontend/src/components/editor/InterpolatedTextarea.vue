<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from "vue";
import { parseInterpolation } from "@/core/utils/placeholders";
import { useSecretsStore } from "@/stores/secrets";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/core/utils/utils";
import Prism from "prismjs";
import { Lock } from "@lucide/vue";
import "prismjs/components/prism-json";

const props = defineProps<{
    modelValue: string;
    variables?: Record<string, string>;
    environmentName?: string;
    placeholder?: string;
    class?: string;
    language?: string;
    showLineNumbers?: boolean;
}>();

const emit = defineEmits(["update:modelValue"]);

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const highlightRef = ref<HTMLDivElement | null>(null);
const textRef = ref<HTMLDivElement | null>(null);
const hotspotRef = ref<HTMLDivElement | null>(null);
const lineNumbersRef = ref<HTMLDivElement | null>(null);

const secretsStore = useSecretsStore();
const tokens = computed(() =>
    parseInterpolation(
        internalValue.value || "",
        props.variables || {},
        secretsStore.secrets,
    ),
);

const lineCount = computed(() => {
    const val = internalValue.value || "";
    return Math.max(1, val.split("\n").length);
});

const internalValue = computed({
    get: () => props.modelValue,
    set: (val) => emit("update:modelValue", val),
});

const syncScroll = () => {
    if (textareaRef.value) {
        const scrollTop = textareaRef.value.scrollTop;
        const scrollLeft = textareaRef.value.scrollLeft;

        if (highlightRef.value) {
            highlightRef.value.scrollTop = scrollTop;
            highlightRef.value.scrollLeft = scrollLeft;
        }
        if (textRef.value) {
            textRef.value.scrollTop = scrollTop;
            textRef.value.scrollLeft = scrollLeft;
        }
        if (hotspotRef.value) {
            hotspotRef.value.scrollTop = scrollTop;
            hotspotRef.value.scrollLeft = scrollLeft;
        }
        if (lineNumbersRef.value) {
            lineNumbersRef.value.scrollTop = scrollTop;
        }
    }
};

const handleHotspotClick = () => {
    textareaRef.value?.focus();
};

onMounted(() => {
    syncScroll();
});

watch(
    () => props.modelValue,
    () => {
        nextTick(syncScroll);
    },
);

const highlightedContent = computed(() => {
    const code = props.modelValue || "";
    const lang =
        props.language === "application/json" ? "json" : props.language;
    if (!lang) return null;
    const grammar = (Prism.languages as any)[lang];
    if (!grammar) return null;

    const variablePattern = {
        pattern: /\{\{.*?\}\}/,
        alias: "interpolated-variable",
    };

    // Use a local grammar so we don't pollute the global Prism state
    const grammarWithVariables = Prism.util.clone(grammar);

    const addVariableToTokens = (tokens: any) => {
        if (!tokens) return;
        const keys = ["string", "property", "attr-value", "url"];
        keys.forEach((key) => {
            if (tokens[key]) {
                if (
                    typeof tokens[key] === "object" &&
                    !Array.isArray(tokens[key])
                ) {
                    if (!tokens[key].inside) tokens[key].inside = {};
                    tokens[key].inside["interpolated-variable"] =
                        variablePattern;
                } else if (Array.isArray(tokens[key])) {
                    tokens[key].forEach((t: any) => {
                        if (typeof t === "object") {
                            if (!t.inside) t.inside = {};
                            t.inside["interpolated-variable"] = variablePattern;
                        }
                    });
                } else {
                    tokens[key] = {
                        pattern: tokens[key],
                        inside: {
                            "interpolated-variable": variablePattern,
                        },
                    };
                }
            }
        });
    };

    addVariableToTokens(grammarWithVariables);

    const finalGrammar = {
        "interpolated-variable": variablePattern,
        ...grammarWithVariables,
    };

    return Prism.highlight(code, finalGrammar, lang);
});
</script>

<template>
    <div
        class="relative w-full group overflow-hidden rounded-md border bg-background transition-shadow focus-within:ring-1 focus-within:ring-primary xrest-editor-container flex"
    >
        <!-- Line Numbers Gutter -->
        <div
            v-if="showLineNumbers !== false"
            ref="lineNumbersRef"
            class="select-none font-mono text-xs leading-5 text-muted-foreground/40 bg-muted/20 border-r p-3 text-right overflow-hidden shrink-0 min-w-[36px]"
        >
            <div v-for="n in lineCount" :key="n" class="h-5 leading-5 font-mono text-xs">
                {{ n }}
            </div>
        </div>

        <!-- Editor Content Box -->
        <div class="relative flex-1 overflow-hidden">
            <!-- 1. Text Layer -->
            <div
                ref="textRef"
                class="absolute inset-0 pointer-events-none p-3 font-mono text-xs leading-5 whitespace-pre break-all overflow-hidden text-foreground"
            >
                <template v-if="highlightedContent">
                    <div
                        v-html="highlightedContent"
                        class="prism-code-inline xrest-editor-highlight"
                    ></div>
                </template>
                <template v-else v-for="(token, i) in tokens" :key="i">
                    <span v-if="token.type === 'text'" class="text-foreground">{{
                        token.content
                    }}</span>
                    <span
                        v-else
                        class="text-transparent selection:text-transparent"
                        >{{ token.content }}</span
                    >
                </template>
            </div>

            <!-- 2. Highlight Layer -->
            <div
                ref="highlightRef"
                class="absolute inset-0 pointer-events-none p-3 font-mono text-xs leading-5 whitespace-pre break-all overflow-hidden"
            >
                <template v-for="(token, i) in tokens" :key="i">
                    <span v-if="token.type === 'text'" class="text-transparent">{{
                        token.content
                    }}</span>
                    <span
                        v-else
                        :class="
                            cn(
                                'px-0.5 rounded transition-all mx-px',
                                token.isSecret
                                    ? 'text-amber-500 bg-amber-500/20'
                                    : token.isValid
                                      ? 'text-primary bg-primary/20'
                                      : 'text-destructive bg-destructive/20 underline decoration-destructive/50 underline-offset-2',
                            )
                        "
                    >
                        {{ token.content }}
                    </span>
                </template>
            </div>

            <!-- 3. Real Textarea -->
            <textarea
                ref="textareaRef"
                v-model="internalValue"
                :placeholder="placeholder"
                :class="
                    cn(
                        'w-full min-h-[150px] bg-transparent border-none p-3 focus:outline-none resize-y relative z-10 font-mono text-xs leading-5 whitespace-pre',
                        'text-foreground caret-foreground selection:bg-primary/30 placeholder:text-muted-foreground/60',
                        props.class,
                    )
                "
                @scroll="syncScroll"
                @input="syncScroll"
                spellcheck="false"
                autocapitalize="off"
                autocorrect="off"
            ></textarea>

            <!-- 4. Hotspot Layer -->
            <div
                ref="hotspotRef"
                class="absolute inset-0 pointer-events-none p-3 font-mono text-xs leading-5 whitespace-pre break-all overflow-hidden z-20"
            >
                <template v-for="(token, i) in tokens" :key="i">
                    <span v-if="token.type === 'text'" class="text-transparent">{{
                        token.content
                    }}</span>
                    <template v-else>
                        <TooltipProvider :delay-duration="0">
                            <Tooltip>
                                <TooltipTrigger as-child>
                                    <span
                                        class="px-0.5 mx-px rounded font-bold text-transparent pointer-events-auto cursor-help"
                                        @mousedown.prevent="handleHotspotClick"
                                    >
                                        {{ token.content }}
                                    </span>
                                </TooltipTrigger>
                                <TooltipContent
                                    side="top"
                                    class="w-fit max-w-[280px] break-all z-[2000] shadow-xl border-primary/20 px-3 py-2"
                                >
                                    <div class="space-y-1.5 p-0.5">
                                        <div
                                            v-if="token.isValid"
                                            class="flex flex-col gap-1"
                                        >
                                            <div
                                                class="flex items-center justify-between gap-4"
                                            >
                                                <div
                                                    class="flex items-center gap-2"
                                                >
                                                    <div
                                                        :class="
                                                            cn(
                                                                'h-1.5 w-1.5 rounded-full animate-pulse',
                                                                token.isSecret
                                                                    ? 'bg-amber-500'
                                                                    : 'bg-primary',
                                                            )
                                                        "
                                                    ></div>
                                                    <span
                                                        class="font-semibold text-muted-foreground text-xs"
                                                        >{{
                                                            token.isSecret
                                                                ? "Secure Secret"
                                                                : "Active Value"
                                                        }}</span
                                                    >
                                                </div>
                                                <span
                                                    v-if="
                                                        !token.isSecret &&
                                                        environmentName
                                                    "
                                                    class="bg-muted px-1 py-0.5 rounded text-muted-foreground font-medium text-[10px] border border-border/50"
                                                    >{{ environmentName }}</span
                                                >
                                                <Lock
                                                    v-if="token.isSecret"
                                                    class="h-3 w-3 text-amber-500"
                                                />
                                            </div>
                                            <div
                                                v-if="token.isSecret"
                                                class="text-amber-500 bg-amber-500/5 p-1.5 rounded border border-amber-500/10 text-xs italic"
                                            >
                                                Value is securely stored and hidden.
                                            </div>
                                            <span
                                                v-else
                                                class="text-primary bg-primary/5 p-1 rounded border border-primary/10 break-all select-all font-mono text-xs"
                                                >{{ token.resolvedValue }}</span
                                            >
                                        </div>
                                        <div
                                            v-else
                                            class="flex flex-col gap-1 text-destructive"
                                        >
                                            <div
                                                class="flex items-center justify-between gap-4"
                                            >
                                                <div
                                                    class="flex items-center gap-2"
                                                >
                                                    <div
                                                        class="h-1.5 w-1.5 rounded-full bg-destructive animate-bounce"
                                                    ></div>
                                                    <span
                                                        class="font-semibold text-xs"
                                                        >Missing Variable</span
                                                    >
                                                </div>
                                                <span
                                                    v-if="environmentName"
                                                    class="bg-destructive/10 px-1 py-0.5 rounded text-destructive font-medium text-[10px] border border-destructive/20"
                                                    >{{ environmentName }}</span
                                                >
                                            </div>
                                            <span class="leading-relaxed text-xs"
                                                >The variable
                                                <code
                                                    class="font-bold underline underline-offset-2"
                                                    >{{ token.content }}</code
                                                >
                                                is not defined.</span
                                            >
                                        </div>
                                    </div>
                                </TooltipContent>
                            </Tooltip>
                        </TooltipProvider>
                    </template>
                </template>
            </div>
        </div>
    </div>
</template>

<style>
/* Base styles for the textarea positioning and syntax highlighting */
.xrest-editor-container textarea,
.xrest-editor-container div {
    font-family:
        "Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco,
        Consolas, "Liberation Mono", "Courier New", monospace;
    letter-spacing: normal;
}

.xrest-editor-highlight {
    color: var(--color-foreground, var(--foreground, #ffffff));
    --token-property: #0550ae;
    --token-string: #116329;
    --token-number: #cf222e;
    --token-boolean: #0550ae;
    --token-punctuation: #57606a;
    --token-comment: #6e7781;
    --token-keyword: #cf222e;
}

.dark .xrest-editor-highlight {
    color: var(--color-foreground, var(--foreground, #ffffff));
    --token-property: #79c0ff;
    --token-string: #a5d6ff;
    --token-number: #ffa657;
    --token-boolean: #79c0ff;
    --token-punctuation: #8b949e;
    --token-comment: #8b949e;
    --token-keyword: #ff7b72;
}

.xrest-editor-highlight .token.interpolated-variable {
    color: transparent !important;
}

.xrest-editor-highlight .token.comment,
.xrest-editor-highlight .token.prolog,
.xrest-editor-highlight .token.doctype,
.xrest-editor-highlight .token.cdata {
    color: var(--token-comment);
}

.xrest-editor-highlight .token.punctuation {
    color: var(--token-punctuation);
}

.xrest-editor-highlight .token.property,
.xrest-editor-highlight .token.keyword,
.xrest-editor-highlight .token.tag {
    color: var(--token-property);
}

.xrest-editor-highlight .token.boolean,
.xrest-editor-highlight .token.constant,
.xrest-editor-highlight .token.number {
    color: var(--token-number);
}

.xrest-editor-highlight .token.string,
.xrest-editor-highlight .token.char,
.xrest-editor-highlight .token.builtin,
.xrest-editor-highlight .token.inserted,
.xrest-editor-highlight .token.attr-value {
    color: var(--token-string);
}

.xrest-editor-highlight .token.operator,
.xrest-editor-highlight .token.entity,
.xrest-editor-highlight .token.url,
.xrest-editor-highlight .token.function {
    color: var(--color-foreground, var(--foreground, #ffffff));
}

.xrest-editor-highlight .token.regex,
.xrest-editor-highlight .token.important,
.xrest-editor-highlight .token.atrule {
    color: var(--token-keyword);
}

.xrest-editor-container div::-webkit-scrollbar {
    display: none;
}

.xrest-editor-container div {
    -ms-overflow-style: none;
    scrollbar-width: none;
}
</style>

