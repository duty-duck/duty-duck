<script lang="ts" setup>
import type { UpdateHttpMonitorCommand } from "bindings/UpdateHttpMonitorCommand";
import type { HttpMonitorFormData } from "~/components/HttpMonitorForm.vue";

const repo = useHttpMonitorRepository();
const route = useRoute();
const router = useRouter();
const localePath = useLocalePath();

const { data: monitorData } = await repo.useHttpMonitor(
  route.params.monitorId as string
);

const onSubmit = async (monitor: HttpMonitorFormData) => {
  const command: UpdateHttpMonitorCommand = {
    isActive: true,
    ...monitor,
  };
  await repo.updateHttpMonitor(route.params.monitorId as string, command);

  router.push(localePath(`/dashboard/monitors/${route.params.monitorId}`));
};
</script>

<template>
  <BContainer v-if="monitorData?.monitor">
    <BBreadcrumb>
      <BBreadcrumbItem :to="localePath('/dashboard')">{{
        $t("dashboard.sidebar.home")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem :to="localePath('/dashboard/monitors')">{{
        $t("dashboard.sidebar.monitors")
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
    />
  </BContainer>
</template>
