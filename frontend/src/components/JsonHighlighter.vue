<script setup lang="ts">
import { computed } from "vue";
import Prism from "prismjs";
import "prismjs/components/prism-json";
import { cn } from "@/lib/utils";

const props = defineProps<{
  code: string;
  language?: string;
  class?: string;
}>();

const highlightedCode = computed(() => {
  if (!props.code) return "";
  try {
    const lang = props.language || "json";
    const grammar = Prism.languages[lang];
    if (!grammar) return props.code;

    // Attempt to format JSON if it's not already formatted
    let codeToHighlight = props.code;
    if (lang === "json") {
      try {
        const parsed = JSON.parse(props.code);
        codeToHighlight = JSON.stringify(parsed, null, 2);
      } catch (e) {
        // Not valid JSON, just highlight as is
      }
    }

    return Prism.highlight(codeToHighlight, grammar, lang);
  } catch (error) {
    console.error("Highlighting error:", error);
    return props.code;
  }
});
</script>

<template>
  <pre
    :class="
      cn(
        'prism-code xrest-highlight font-mono whitespace-pre p-3 rounded-md bg-background shadow-sm leading-relaxed overflow-auto border',
        props.class,
      )
    "
  ><code v-html="highlightedCode"></code></pre>
</template>

<style>
.xrest-highlight {
  --code-foreground: #24292f;
  --token-property: #0550ae;
  --token-string: #116329;
  --token-number: #cf222e;
  --token-boolean: #0550ae;
  --token-punctuation: #57606a;
  --token-comment: #6e7781;
  --token-keyword: #cf222e;
  --token-function: #8250df;
  --token-null: #0550ae;
  
  color: var(--code-foreground);
  line-height: 1.6;
  font-size: 0.875rem;
}

.dark .xrest-highlight {
  --code-foreground: #c9d1d9;
  --token-property: #79c0ff;
  --token-string: #a5d6ff;
  --token-number: #ffa657;
  --token-boolean: #79c0ff;
  --token-punctuation: #8b949e;
  --token-comment: #8b949e;
  --token-keyword: #ff7b72;
  --token-function: #d2a8ff;
  --token-null: #79c0ff;
}

.xrest-highlight .token.comment,
.xrest-highlight .token.prolog,
.xrest-highlight .token.doctype,
.xrest-highlight .token.cdata {
  color: var(--token-comment);
  font-style: italic;
}

.xrest-highlight .token.punctuation {
  color: var(--token-punctuation);
}

.xrest-highlight .token.property,
.xrest-highlight .token.tag,
.xrest-highlight .token.constant,
.xrest-highlight .token.symbol,
.xrest-highlight .token.deleted {
  color: var(--token-property);
  font-weight: 500;
}

.xrest-highlight .token.boolean,
.xrest-highlight .token.number {
  color: var(--token-number);
}

.xrest-highlight .token.selector,
.xrest-highlight .token.attr-name,
.xrest-highlight .token.string,
.xrest-highlight .token.char,
.xrest-highlight .token.builtin,
.xrest-highlight .token.inserted {
  color: var(--token-string);
}

.xrest-highlight .token.operator,
.xrest-highlight .token.entity,
.xrest-highlight .token.url {
  color: var(--code-foreground);
}

.xrest-highlight .token.keyword {
  color: var(--token-keyword);
}

.xrest-highlight .token.function {
  color: var(--token-function);
}

.xrest-highlight .token.important,
.xrest-highlight .token.bold {
  font-weight: 600;
}

.xrest-highlight .token.italic {
  font-style: italic;
}

/* JSON Specific Adjustments */
.language-json.xrest-highlight .token.property {
  color: var(--token-property);
}

.language-json.xrest-highlight .token.boolean {
  color: var(--token-boolean);
  font-weight: 600;
}

.language-json.xrest-highlight .token.null {
  color: var(--token-null);
  font-weight: 600;
}
</style>
