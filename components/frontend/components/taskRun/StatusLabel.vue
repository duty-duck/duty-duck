<script setup lang="ts">
import type { TaskRunStatus } from "bindings/TaskRunStatus";
import type { BaseColorVariant } from "bootstrap-vue-next";

const props = defineProps<{
  status: TaskRunStatus;
}>();

const variants: Record<TaskRunStatus, keyof BaseColorVariant> = {
  "running": "info",
  "failed": "danger",
  "dead": "danger",
  "finished": "primary",
  "aborted": "warning",
}
</script>

<template>
  <BBadge :variant="variants[status]" class="rounded-pill text-white">
    {{ $t(`dashboard.taskRunStatus.${status}`) }}
  </BBadge>
</template>