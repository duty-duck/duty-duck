<script setup lang="ts">
  const localePath = useLocalePath();
</script>
<template>
  <BButton class="icon-link" variant="primary" size="sm" icon :to="localePath('/dashboard/monitors/new')">
    <Icon name="ph:plus-bold" />
    {{ $t('dashboard.monitors.createMonitorBtn') }}
  </BButton>
</template>
