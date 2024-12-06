<script setup lang="ts">
import type { TaskStatus } from "bindings/TaskStatus";
import type { BaseColorVariant } from "bootstrap-vue-next";

const props = defineProps<{
  status: TaskStatus;
}>();

const variants: Record<TaskStatus, keyof BaseColorVariant> = {
  "due": "info",
  "failing": "danger",
  "healthy": "success",
  "late": "warning",
  "running": "info",
  "absent": "danger",
}
</script>

<template>
  <BBadge :variant="variants[status]" class="rounded-pill text-white">
    {{ $t(`dashboard.taskStatus.${status}`) }}
  </BBadge>
</template>
