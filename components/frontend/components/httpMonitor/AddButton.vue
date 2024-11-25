<script setup lang="ts">
const localePath = useLocalePath();
const { userHasPermissionComputed } = await useAuth();
const canCreateMonitors = userHasPermissionComputed('writeHttpMonitors');
</script>
<template>
  <BButton class="icon-link" variant="primary" size="sm" icon :to="localePath('/dashboard/httpMonitors/new')"
    v-if="canCreateMonitors">
    <Icon name="ph:plus-bold" />
    {{ $t('dashboard.monitors.createMonitorBtn') }}
  </BButton>
</template>
