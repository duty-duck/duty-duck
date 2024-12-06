<script setup lang="ts">
import type { TaskRun} from "bindings/TaskRun";
const { taskRuns} = defineProps<{
    taskRuns: TaskRun[]
}>();

</script>

<template>
    <div class="task-runs-grid">
        <div v-for="t in taskRuns" :key="t.startedAt" :class="{
            'bg-success': t.status == 'finished',
            'bg-danger': t.status == 'failed' || t.status == 'dead',
            'bg-secondary': t.status == 'aborted',
            'bg-info': t.status == 'running',
        }">
            <div class="tooltip"
                v-b-tooltip.hover.bottom="`${$d(new Date(t.startedAt!), 'long')} ${$t(`dashboard.taskRunStatus.${t.status}`)}`" />
        </div>
    </div>
</template>

<style lang="scss" scoped>
$square-size: 12px;

.task-runs-grid {
    display: grid;
    grid-template-columns: repeat(12, $square-size);
    grid-auto-rows: $square-size;
    grid-gap: 3px;

    // grid squares
    >div {
        border-radius: 2px;

        // tooltips are set on a div inside the grid square to avoid interfering with the grid layout
        >.tooltip {
            width: $square-size;
            height: $square-size;
        }
    }
}
</style>