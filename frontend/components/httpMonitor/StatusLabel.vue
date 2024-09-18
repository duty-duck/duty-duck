<script setup lang="ts">
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const props = defineProps<{
  status: HttpMonitorStatus;
}>();
</script>

<template>
  <span :class="{
    'text-danger': props.status == 'down',
    'text-warning': props.status == 'suspicious',
    'text-info': props.status == 'recovering',
    'text-secondary': props.status == 'unknown' || props.status == 'inactive',
    'text-success': props.status == 'up'
  }">
    {{ $t(`dashboard.monitorStatus.${status}`) }}
  </span>
</template>
