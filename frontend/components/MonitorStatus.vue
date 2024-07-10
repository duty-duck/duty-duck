<script setup lang="ts">
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const props = defineProps<{
  status: HttpMonitorStatus;
}>();

const label = computed(() => {
  if (props.status == "unknown") {
    return "PENDING";
  }
  if (props.status == "up") {
    return "HEALTHY";
  }

  return props.status.toUpperCase();
});

const icon = computed(() => {
  if (props.status == "unknown" || props.status == "recovering") {
    return "ph:circle-duotone";
  }
  if (props.status == "down" || props.status == "suspicious") {
    return "ph:warning-circle-duotone";
  }
  if (props.status == "inactive") {
    return "ph:pause-circle-duotone"
  }

  return "ph:check-circle-duotone"
});
</script>

<template>
  <span
  class="icon-link"
    :class="{
      'text-danger': props.status == 'down',
      'text-warning': props.status == 'suspicious',
      'text-info': props.status == 'recovering',
      'text-secondary': props.status == 'unknown' || props.status == 'inactive',
      'text-success': props.status == 'up'
    }"
    >
    <Icon :name="icon" size="2rem" />
    {{ label }}</span
  >
</template>
