<script lang="ts" setup>
import type { UpdateHttpMonitorCommand } from "bindings/UpdateHttpMonitorCommand";
import type { HttpMonitorFormData } from "~/components/httpMonitor/Form.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("writeHttpMonitors");

const repo = await useHttpMonitorRepository();
const route = useRoute();
const localePath = useLocalePath();

const { data: monitorData } = await repo.useHttpMonitor(
  route.params.monitorId as string
);

const onSubmit = async (monitor: HttpMonitorFormData) => {
  const command: UpdateHttpMonitorCommand = {
    isActive: true,
    ...monitor.notificationSettings,
    ...monitor,
  };
  await repo.updateHttpMonitor(route.params.monitorId as string, command);

  navigateTo(localePath(`/dashboard/httpMonitors/${route.params.monitorId}`));
};
</script>

<template>
  <BContainer v-if="monitorData?.monitor">
    <BBreadcrumb>
      <BBreadcrumbItem :to="localePath('/dashboard')">{{
        $t("dashboard.mainSidebar.home")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem :to="localePath('/dashboard/httpMonitors')">{{
        $t("dashboard.mainSidebar.monitors")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem active>
        {{ $t("dashboard.monitors.edit") }}
      </BBreadcrumbItem>
    </BBreadcrumb>
    <HttpMonitorForm
      @submit="onSubmit"
      :url="monitorData.monitor.url"
      :tags="monitorData.monitor.tags"
      :interval-seconds="monitorData.monitor.intervalSeconds"
      :downtime-confirmation-threshold="monitorData.monitor.downtimeConfirmationThreshold"
      :recovery-confirmation-threshold="monitorData.monitor.recoveryConfirmationThreshold"
      :notification-settings="{
        pushNotificationEnabled: monitorData.monitor.pushNotificationEnabled,
        emailNotificationEnabled: monitorData.monitor.emailNotificationEnabled,
        smsNotificationEnabled: monitorData.monitor.smsNotificationEnabled,
      }"
    />
  </BContainer>
</template>
