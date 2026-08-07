<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { EditorState, Extension } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine, Decoration, DecorationSet, WidgetType, ViewPlugin, ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { json } from "@codemirror/lang-json";
import { xml } from "@codemirror/lang-xml";
import { oneDark } from "@codemirror/theme-one-dark";
import { useSecretsStore } from "@/stores/secrets";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    readOnly?: boolean;
    language?: string;
    variables?: Record<string, string>;
    environmentName?: string;
    placeholder?: string;
    class?: string;
  }>(),
  {
    readOnly: false,
    language: "json",
  }
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const editorRef = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;

const secretsStore = useSecretsStore();

// --- Variable Token Decorator for {{variable}} ---
class VariableWidget extends WidgetType {
  constructor(
    readonly name: string,
    readonly isValid: boolean,
    readonly isSecret: boolean,
    readonly resolvedValue?: string,
    readonly envName?: string
  ) {
    super();
  }

  toDOM() {
    const span = document.createElement("span");
    span.className = `inline-flex items-center px-1 rounded text-xs font-mono font-semibold mx-0.5 select-none cursor-help ${
      this.isSecret
        ? "bg-amber-500/20 text-amber-500 border border-amber-500/30"
        : this.isValid
        ? "bg-primary/20 text-primary border border-primary/30"
        : "bg-destructive/20 text-destructive border border-destructive/30 underline decoration-destructive"
    }`;
    span.textContent = `{{${this.name}}}`;
    span.title = this.isSecret
      ? "Secure Secret (Hidden)"
      : this.isValid
      ? `Resolved: ${this.resolvedValue || ""} (${this.envName || "Active Env"})`
      : `Missing Variable: {{${this.name}}}`;
    return span;
  }
}

const buildVariableDecorations = (docText: string) => {
  const builder = new Array<{ from: number; to: number; decoration: Decoration }>();
  const regex = /\{\{\s*([\w.-]+)\s*\}\}/g;
  let match;

  const vars = props.variables || {};
  const secrets = secretsStore.secrets || [];

  while ((match = regex.exec(docText)) !== null) {
    const from = match.index;
    const to = from + match[0].length;
    const name = match[1];

    const isSecret = name.startsWith("secret.") || secrets.includes(name);
    const resolvedValue = vars[name];
    const isValid = isSecret || resolvedValue !== undefined;

    const widget = Decoration.replace({
      widget: new VariableWidget(name, isValid, isSecret, resolvedValue, props.environmentName),
    });
    builder.push({ from, to, decoration: widget });
  }

  return builder;
};

const variablePlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      const doc = view.state.doc.toString();
      const decos = buildVariableDecorations(doc);
      this.decorations = Decoration.set(
        decos.map((d) => d.decoration.range(d.from, d.to)),
        true
      );
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        const doc = update.view.state.doc.toString();
        const decos = buildVariableDecorations(doc);
        this.decorations = Decoration.set(
          decos.map((d) => d.decoration.range(d.from, d.to)),
          true
        );
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);

// Determine CodeMirror extensions
const getLanguageExtension = (lang?: string): Extension => {
  const l = (lang || "").toLowerCase();
  if (l.includes("json")) return json();
  if (l.includes("xml") || l.includes("html")) return xml();
  return [];
};

// Custom theme tuning for dark / light modes
const lightTheme = EditorView.theme({
  "&": {
    fontSize: "12px",
    fontFamily: '"Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    backgroundColor: "transparent",
    color: "hsl(var(--foreground))",
  },
  ".cm-content": {
    fontFamily: '"Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    padding: "8px 0",
  },
  ".cm-gutters": {
    backgroundColor: "hsl(var(--muted) / 0.3)",
    color: "hsl(var(--muted-foreground) / 0.5)",
    borderRight: "1px solid hsl(var(--border))",
    fontSize: "11px",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "hsl(var(--muted) / 0.5)",
    color: "hsl(var(--foreground))",
  },
  ".cm-activeLine": {
    backgroundColor: "hsl(var(--muted) / 0.15)",
  },
  "&.cm-focused": {
    outline: "none",
  },
});

const darkTheme = EditorView.theme({
  "&": {
    fontSize: "12px",
    fontFamily: '"Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    backgroundColor: "transparent",
    color: "hsl(var(--foreground))",
  },
  ".cm-content": {
    fontFamily: '"Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    padding: "8px 0",
  },
  ".cm-gutters": {
    backgroundColor: "hsl(var(--muted) / 0.2)",
    color: "hsl(var(--muted-foreground) / 0.5)",
    borderRight: "1px solid hsl(var(--border) / 0.5)",
    fontSize: "11px",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "hsl(var(--muted) / 0.4)",
    color: "hsl(var(--foreground))",
  },
  ".cm-activeLine": {
    backgroundColor: "hsl(var(--muted) / 0.2)",
  },
  "&.cm-focused": {
    outline: "none",
  },
});

const isDarkMode = () => document.documentElement.classList.contains("dark");

const createEditorState = () => {
  const extensions: Extension[] = [
    lineNumbers(),
    highlightActiveLine(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    getLanguageExtension(props.language),
    variablePlugin,
    isDarkMode() ? [oneDark, darkTheme] : lightTheme,
    EditorView.lineWrapping,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const val = update.state.doc.toString();
        emit("update:modelValue", val);
      }
    }),
  ];

  if (props.readOnly) {
    extensions.push(EditorState.readOnly.of(true));
  }

  return EditorState.create({
    doc: props.modelValue || "",
    extensions,
  });
};

onMounted(() => {
  if (!editorRef.value) return;

  const state = createEditorState();
  view = new EditorView({
    state,
    parent: editorRef.value,
  });
});

onUnmounted(() => {
  if (view) {
    view.destroy();
    view = null;
  }
});

// Watch for external modelValue changes
watch(
  () => props.modelValue,
  (newVal) => {
    if (view && view.state.doc.toString() !== newVal) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: newVal || "",
        },
      });
    }
  }
);

// Watch for language changes
watch(
  () => props.language,
  () => {
    if (view) {
      view.setState(createEditorState());
    }
  }
);
</script>

<template>
  <div
    ref="editorRef"
    :class="[
      'w-full h-full overflow-hidden rounded-md border bg-card text-foreground focus-within:ring-1 focus-within:ring-primary shadow-xs',
      props.class,
    ]"
  ></div>
</template>

<style>
.cm-editor {
  height: 100%;
}
.cm-scroller {
  overflow: auto;
  font-family: "Roboto Mono Variable", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
}
</style>
