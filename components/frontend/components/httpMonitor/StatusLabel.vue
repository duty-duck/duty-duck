<script setup lang="ts">
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import type { BaseColorVariant } from "bootstrap-vue-next";

const props = defineProps<{
  status: HttpMonitorStatus;
}>();

const variants: Record<HttpMonitorStatus, keyof BaseColorVariant> = {
  "up": "success",
  "down": "danger",
  "recovering": "info",
  "suspicious": "warning",
  "inactive": "secondary",
  "unknown": "secondary"
}
</script>

<template>
  <BBadge :variant="variants[status]" class="rounded-pill text-white">
    {{ $t(`dashboard.monitorStatus.${status}`) }}
  </BBadge>
</template>
