<script setup lang="ts">
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const props = defineProps<{
  status: HttpMonitorStatus;
}>();

const label = computed(() => {
  if (props.status == "unknown") {
    return "Pending";
  }
  if (props.status == "up") {
    return "Healthy";
  }

  return props.status;
});
</script>

<template>
  <span style="text-transform: capitalize;" :class="{
    'text-danger': props.status == 'down',
    'text-warning': props.status == 'suspicious',
    'text-info': props.status == 'recovering',
    'text-secondary': props.status == 'unknown' || props.status == 'inactive',
    'text-success': props.status == 'up'
  }">
    {{ label }}
  </span>
</template>
