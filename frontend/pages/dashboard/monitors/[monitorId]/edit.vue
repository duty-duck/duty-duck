<script lang="ts" setup>
const localePath = useLocalePath();
const repo = useHttpMonitorRepository();
const route = useRoute();

const { data: monitorData } = await repo.useHttpMonitor(
  route.params.monitorId as string
);
</script>

<template>
  <BContainer v-if="monitorData?.monitor">
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard">{{
        $t("dashboard.sidebar.home")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem to="/dashboard/monitors">{{
        $t("dashboard.sidebar.monitors")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem active>
        {{ $t("dashboard.monitors.edit") }}
      </BBreadcrumbItem>
    </BBreadcrumb>
    <HttpMonitorForm
      :url="monitorData.monitor.url"
      :tags="monitorData.monitor.tags"
      :interval-seconds="monitorData.monitor.intervalSeconds"
    />
  </BContainer>
</template>
