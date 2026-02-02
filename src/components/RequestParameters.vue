<script setup lang="ts">
import { X, Plus } from "lucide-vue-next";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import InterpolatedInput from "./InterpolatedInput.vue";

const props = defineProps<{
  items: any[];
  variables: Record<string, string>;
  environmentName: string;
}>();

const addRow = () => {
  props.items.push({ enabled: true, name: "", value: "" });
};

const removeRow = (index: number) => {
  if (props.items.length > 1) {
    props.items.splice(index, 1);
  } else {
    props.items[0] = { enabled: true, name: "", value: "" };
  }
};
</script>

<template>
  <div class="space-y-2">
    <div class="border rounded-md overflow-hidden bg-card">
      <Table>
        <TableHeader>
          <TableRow class="hover:bg-transparent border-b">
            <TableHead class="w-10 px-2"></TableHead>
            <TableHead class="font-bold h-8">Key</TableHead>
            <TableHead class="font-bold h-8 border-l">Value</TableHead>
            <TableHead class="w-10 px-2 border-l"></TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, rIdx) in items" :key="rIdx" class="group h-8">
            <TableCell class="p-0 text-center">
              <Checkbox
                :checked="row.enabled"
                @update:checked="row.enabled = $event"
                class="scale-75"
              />
            </TableCell>
            <TableCell class="p-0">
              <InterpolatedInput
                v-model="row.name"
                :variables="variables"
                :environment-name="environmentName"
                class="h-8 border-none focus-visible:ring-0 shadow-none px-3"
                placeholder="Key"
              />
            </TableCell>
            <TableCell class="p-0 border-l">
              <InterpolatedInput
                v-model="row.value"
                :variables="variables"
                :environment-name="environmentName"
                class="h-8 border-none focus-visible:ring-0 shadow-none px-3"
                placeholder="Value"
              />
            </TableCell>
            <TableCell
              class="p-0 text-center opacity-0 group-hover:opacity-100 transition-opacity border-l"
            >
              <button
                @click="removeRow(Number(rIdx))"
                class="p-1 hover:text-destructive"
              >
                <X class="h-3 w-3" />
              </button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
    <Button
      variant="ghost"
      size="sm"
      class="mt-2 h-7 gap-1 text-primary hover:bg-primary/5"
      @click="addRow"
    >
      <Plus class="h-3 w-3" /> Add Row
    </Button>
  </div>
</template>
