<script setup lang="ts">
import { ref, computed } from "vue";
import { Search, ArrowUpDown, Table as TableIcon } from "@lucide/vue";

const props = defineProps<{
    content: string;
}>();

const searchQuery = ref("");
const sortCol = ref<number | null>(null);
const sortAsc = ref(true);

const parsedData = computed(() => {
    if (!props.content) return { headers: [], rows: [] };
    const lines = props.content.trim().split(/\r?\n/);
    if (lines.length === 0) return { headers: [], rows: [] };

    const delimiter = props.content.includes("\t") ? "\t" : ",";
    const headers = lines[0].split(delimiter).map((h) => h.trim().replace(/^["']|["']$/g, ""));
    const rows = lines.slice(1).map((line) => line.split(delimiter).map((cell) => cell.trim().replace(/^["']|["']$/g, "")));

    return { headers, rows };
});

const filteredRows = computed(() => {
    let rows = parsedData.value.rows;
    if (searchQuery.value.trim()) {
        const q = searchQuery.value.toLowerCase();
        rows = rows.filter((row) => row.some((cell) => cell.toLowerCase().includes(q)));
    }

    if (sortCol.value !== null) {
        const colIdx = sortCol.value;
        rows = [...rows].sort((a, b) => {
            const valA = a[colIdx] || "";
            const valB = b[colIdx] || "";
            return sortAsc.value ? valA.localeCompare(valB) : valB.localeCompare(valA);
        });
    }

    return rows;
});

const sortBy = (colIdx: number) => {
    if (sortCol.value === colIdx) {
        sortAsc.value = !sortAsc.value;
    } else {
        sortCol.value = colIdx;
        sortAsc.value = true;
    }
};
</script>

<template>
    <div class="flex flex-col h-full bg-background border rounded-md overflow-hidden">
        <!-- Search & Info Bar -->
        <div class="flex items-center justify-between px-3 py-1.5 border-b bg-muted/40 text-xs">
            <div class="flex items-center gap-2">
                <TableIcon class="h-3.5 w-3.5 text-muted-foreground" />
                <span class="font-medium text-foreground">Tabular Data Grid</span>
                <span class="text-muted-foreground">({{ parsedData.rows.length }} rows)</span>
            </div>

            <div class="relative w-48">
                <Search class="absolute left-2 top-2 h-3.5 w-3.5 text-muted-foreground" />
                <input
                    v-model="searchQuery"
                    type="text"
                    placeholder="Search rows..."
                    class="w-full pl-7 pr-2 py-1 text-xs bg-background border rounded focus:outline-none focus:ring-1 focus:ring-primary"
                />
            </div>
        </div>

        <!-- Table Grid -->
        <div class="flex-1 overflow-auto p-2">
            <table class="w-full text-xs text-left border-collapse">
                <thead class="bg-muted/60 sticky top-0 font-semibold border-b">
                    <tr>
                        <th
                            v-for="(header, idx) in parsedData.headers"
                            :key="idx"
                            @click="sortBy(idx)"
                            class="p-2 cursor-pointer hover:bg-muted select-none"
                        >
                            <div class="flex items-center gap-1">
                                <span>{{ header }}</span>
                                <ArrowUpDown class="h-3 w-3 text-muted-foreground/60" />
                            </div>
                        </th>
                    </tr>
                </thead>
                <tbody class="divide-y">
                    <tr
                        v-for="(row, rIdx) in filteredRows"
                        :key="rIdx"
                        class="hover:bg-muted/40 transition-colors"
                    >
                        <td v-for="(cell, cIdx) in row" :key="cIdx" class="p-2 font-mono break-all">
                            {{ cell }}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</template>
