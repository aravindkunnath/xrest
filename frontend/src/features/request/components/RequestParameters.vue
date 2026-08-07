<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { RequestParams, serializeForBulkEdit } from "@/core/utils/request-param-utils.ts";
import { cn } from "@/core/utils/utils";
import { Edit3, Plus, Table as TableIcon, Trash2, X } from "@lucide/vue";
import { ref, watch } from "vue";
import InterpolatedInput from "@/components/editor/InterpolatedInput.vue";

const props = defineProps<{
    items: RequestParams[];
    variables: Record<string, string>;
    environmentName: string;
}>();

const isBulkEdit = ref(false);
const bulkText = ref("");


const hadOnlyEnabled = ref(false);
const hadOnlyName = ref(false);

const parseBulkText = (text: string) => {
    const parsedItems: any[] = [];

    for (const rawLine of text.split("\n")) {
        const line = rawLine.trim();
        if (!line) continue;

        let enabled = true;
        let processLine = line;

        if (processLine.startsWith("//")) {
            enabled = false;
            processLine = processLine.slice(2).trim();
        } else if (processLine.startsWith("#")) {
            enabled = false;
            processLine = processLine.slice(1).trim();
        }

        const separatorIndex = processLine.search(/[:=]/);
        const key = separatorIndex === -1
            ? processLine.trim()
            : processLine.slice(0, separatorIndex).trim();
        const value = separatorIndex === -1
            ? ""
            : processLine.slice(separatorIndex + 1).trim();

        if (hadOnlyEnabled.value && hadOnlyName.value) {
            parsedItems.push({ enabled, name: key, value });
        } else if (hadOnlyName.value) {
            parsedItems.push({ enabled, name: key, value });
        } else if (hadOnlyEnabled.value) {
            parsedItems.push({ enabled, key, value });
        } else {
            parsedItems.push({ isEnabled: enabled, enabled, key, name: key, value });
        }
    }

    return parsedItems.length > 0
        ? parsedItems
        : [{ isEnabled: true, enabled: true, key: "", name: "", value: "" }];
};

const normalizeItems = () => {
    if (!props.items || props.items.length === 0) {
        props.items.push((hadOnlyEnabled.value || hadOnlyName.value) ? { enabled: true, name: "", value: "" } : { isEnabled: true, key: "", value: "" });
        return;
    }

    hadOnlyEnabled.value = props.items.every((item: any) => item.isEnabled === undefined && item.enabled !== undefined);
    hadOnlyName.value = props.items.every((item: any) => item.key === undefined && item.name !== undefined);
};
normalizeItems();

const addRow = () => {
    props.items.push((hadOnlyEnabled.value || hadOnlyName.value) ? { enabled: true, name: "", value: "" } : { isEnabled: true, key: "", value: "" });
};

const removeRow = (index: number) => {
    if (props.items.length > 1) {
        props.items.splice(index, 1);
    } else {
        props.items[0] = (hadOnlyEnabled.value || hadOnlyName.value) ? { enabled: true, name: "", value: "" } : { isEnabled: true, key: "", value: "" };
    }
};

const clearAll = () => {
    props.items.splice(0, props.items.length, (hadOnlyEnabled.value || hadOnlyName.value) ? { enabled: true, name: "", value: "" } : { isEnabled: true, key: "", value: "" });
    if (isBulkEdit.value) {
        bulkText.value = "";
    }
};

const toggleBulkEdit = () => {
    if (isBulkEdit.value) {
        const newItems = parseBulkText(bulkText.value);
        if (hadOnlyEnabled.value || hadOnlyName.value) {
            newItems.forEach((item: any) => {
                delete item.isEnabled;
                delete item.key;
            });
        }
        props.items.splice(0, props.items.length, ...newItems);
        isBulkEdit.value = false;
    } else {
        bulkText.value = serializeForBulkEdit(props.items);
        isBulkEdit.value = true;
    }
};

// Sync bulk edit text if items change externally while in bulk mode
watch(
    () => props.items,
    () => {
        normalizeItems();
        if (isBulkEdit.value) {
            bulkText.value = serializeForBulkEdit(props.items);
        }
    },
    { deep: true, immediate: true }
);
</script>

<template>
    <div class="space-y-2 p-4">
        <!-- Header Toolbar -->
        <div class="flex items-center justify-between pb-1">
            <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-foreground/80 tracking-tight">
                    Parameters
                </span>
            </div>

            <div class="flex items-center gap-1">
                <!-- <Button
                    variant="ghost"
                    size="sm"
                    class="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground"
                    @click="toggleAll"
                    title="Toggle all checkboxes"
                >
                    <CheckSquare class="h-3.5 w-3.5" />
                    <span class="hidden sm:inline">Toggle All</span>
                </Button> -->
                <Button variant="ghost" size="sm"
                    class="h-7 text-xs px-2 gap-1 text-muted-foreground hover:text-foreground" @click="toggleBulkEdit">
                    <component :is="isBulkEdit ? TableIcon : Edit3" class="h-3.5 w-3.5" />
                    {{ isBulkEdit ? "Key-Value Edit" : "Bulk Edit" }}
                </Button>
                <Button variant="ghost" size="sm"
                    class="h-7 text-xs px-2 text-muted-foreground hover:text-destructive gap-1" @click="clearAll"
                    title="Clear all parameters">
                    <Trash2 class="h-3.5 w-3.5" />
                    <span class="hidden sm:inline">Clear</span>
                </Button>
            </div>
        </div>

        <!-- Bulk Edit Mode -->
        <div v-if="isBulkEdit" class="space-y-2 animate-in fade-in-50 duration-150">
            <textarea v-model="bulkText" rows="6" placeholder="Key: Value (one per line, prefix with // to disable)"
                class="w-full font-mono text-xs p-3 bg-card border rounded-md focus:outline-none focus:ring-1 focus:ring-primary resize-y leading-relaxed text-foreground"></textarea>
            <div class="flex items-center justify-between text-[11px] text-muted-foreground px-1">
                <span>Format: <code>key: value</code> or <code>key=value</code></span>
                <Button size="sm" class="h-7 text-xs px-3" @click="toggleBulkEdit">
                    Done Editing
                </Button>
            </div>
        </div>

        <!-- Key-Value Table View -->
        <div v-else class="border rounded-md overflow-hidden bg-card shadow-sm">
            <Table>
                <TableHeader class="bg-muted/30">
                    <TableRow class="hover:bg-transparent border-b">
                        <TableHead class="w-9 px-2 text-center h-8">
                            <span class="sr-only">Status</span>
                        </TableHead>
                        <TableHead
                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-wider h-8 px-3">
                            Key
                        </TableHead>
                        <TableHead
                            class="text-[10px] uppercase font-bold text-muted-foreground tracking-wider h-8 border-l px-3">
                            Value
                        </TableHead>
                        <TableHead class="w-9 px-0 text-center border-l h-8">
                            <span class="sr-only">Actions</span>
                        </TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <TableRow v-for="(row, rIdx) in items" :key="rIdx" :class="[
                        'group h-8 transition-colors border-b last:border-b-0',
                        !(row.isEnabled ?? row.enabled) ? 'bg-muted/10 text-muted-foreground/60' : 'hover:bg-muted/20'
                    ]">
                        <TableCell class="p-0 text-center w-9 align-middle">
                            <div class="flex items-center justify-center h-8">
                                <Checkbox :checked="Boolean(row.isEnabled ?? row.enabled)" @update:checked="(val: any) => { if (row.isEnabled !== undefined) row.isEnabled = Boolean(val); if (row.enabled !== undefined) row.enabled = Boolean(val); }" class="scale-75" />
                            </div>
                        </TableCell>
                        <TableCell class="p-0 align-middle">
                            <InterpolatedInput :model-value="(row.key ?? row.name) || ''" @update:model-value="(val: string) => { if (row.key !== undefined) row.key = val; if (row.name !== undefined) row.name = val; }" :variables="variables"
                                :environment-name="environmentName" :class="cn(
                                    'h-8 border-none focus-visible:ring-0 shadow-none px-3 font-mono text-xs bg-transparent',
                                    !(row.isEnabled ?? row.enabled) && 'opacity-50 line-through'
                                )" placeholder="Key" />
                        </TableCell>
                        <TableCell class="p-0 border-l align-middle">
                            <InterpolatedInput v-model="row.value" :variables="variables"
                                :environment-name="environmentName" :class="cn(
                                    'h-8 border-none focus-visible:ring-0 shadow-none px-3 font-mono text-xs bg-transparent',
                                    !(row.isEnabled ?? row.enabled) && 'opacity-50 line-through'
                                )" placeholder="Value" />
                        </TableCell>
                        <TableCell class="p-0 text-center border-l w-9 align-middle">
                            <div
                                class="flex items-center justify-center h-8 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button @click="removeRow(Number(rIdx))"
                                    class="p-1 text-muted-foreground hover:text-destructive rounded-sm hover:bg-muted transition-colors"
                                    title="Delete row">
                                    <X class="h-3.5 w-3.5" />
                                </button>
                            </div>
                        </TableCell>
                    </TableRow>
                </TableBody>
            </Table>
        </div>

        <div v-if="!isBulkEdit" class="flex items-center justify-between pt-1">
            <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs text-primary hover:bg-primary/5 px-2"
                @click="addRow">
                <Plus class="h-3.5 w-3.5" /> Add Row
            </Button>
        </div>
    </div>
</template>
