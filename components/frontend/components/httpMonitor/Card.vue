<script setup lang="ts">
import type { HttpMonitor } from "bindings/HttpMonitor";
const localePath = useLocalePath();
const { monitor, animated } = defineProps<{ monitor: HttpMonitor, animated?: boolean }>();
</script>

<template>
  <NuxtLink :to="localePath(`/dashboard/httpMonitors/${monitor.id}`)" class="card shadow-sm"
    :class="{ 'slide-up-fade-in': animated }">
    <BCardBody class="d-flex align-items-center px-2">
      <HttpMonitorStatusIcon :status="monitor.status" class="mx-4 mx-lg-5" />
      <div class="flex-grow-1">
        <div class="h6">
          {{ monitor.url }}
        </div>
        <HttpMonitorStatusLabel :status="monitor.status" />
        <span v-show="monitor.lastPingAt" class="small text-secondary">
          &nbsp;
          {{ $t('dashboard.monitors.lastCheckedOn', { date: $d(new Date(monitor.lastPingAt!), 'long') }) }}
        </span>
        <div class="d-flex gap-1 mt-2">
          <BBadge v-for="t in monitor.tags" variant="light">{{ t }}</BBadge>
        </div>
      </div>
      <BButtonToolbar class="align-self-start">
        <NuxtLink class="btn btn-sm btn-light icon-link">
          <Icon name="ph:eye-fill" />
          {{ $t('dashboard.monitors.details') }}
        </NuxtLink>
      </BButtonToolbar>
    </BCardBody>
  </NuxtLink>
</template>

<style scoped lang="scss">
.btn-toolbar {
  display: none;
}

.card {
  cursor: pointer;
  text-decoration: inherit;
}

.card:hover {
  .btn-toolbar {
    display: unset;
  }
}

@for $i from 1 through 10 {
  @keyframes slideUpFadeIn#{$i} {
    0% {
      opacity: 0;
      transform: translateY(30px);
    }

    #{$i* 10 + "%"} {
      opacity: 0;
      transform: translateY(30px);
    }

    100% {
      opacity: 1;
      transform: translateY(0);
    }
  }
}

@keyframes slideUpFadeIn {
  from {
    opacity: 0;
    transform: translateY(30px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.slide-up-fade-in {
  @for $i from 1 through 10 {
    &:nth-child(#{$i}n) {
      animation: slideUpFadeIn#{$i} 0.3s ease-out;
    }
  }

}
</style>