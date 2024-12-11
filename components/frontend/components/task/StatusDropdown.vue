<script lang="ts">
import type { TaskStatus } from 'bindings/TaskStatus';
export const allStatuses: TaskStatus[] = [
    "due",
    "failing",
    "healthy",
    "late",
    "running",
    "absent",
];
</script>

<script setup lang="ts">
const model = defineModel<TaskStatus[]>();
</script>

<template>
    <BDropdown variant="outline-secondary" toggle-class="d-flex align-items-center gap-1">
        <template #button-content>
            <Icon name="ph:funnel" size="1.3rem" />
            {{ $t('dashboard.tasks.taskStatus') }}
            <span v-if="model!.length < allStatuses.length">({{ model?.length }})</span>
        </template>
        <BDropdownItem v-for="s in allStatuses">
            <input :id="`${s}-checkbox`" :key="s" type="checkbox" v-model="model" :value="s" />
            <label :for="`${s}-checkbox`" style="width: 100%; text-transform: capitalize;" class="ps-2">{{
                $t(`dashboard.taskStatus.${s}`) }}</label>
        </BDropdownItem>
    </BDropdown>
</template>