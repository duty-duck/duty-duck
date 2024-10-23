<script lang="ts" setup>
import type { HttpMonitorFormData } from "~/components/httpMonitor/Form.vue";
import { usePermissionGrant } from "~/composables/authComposables";

await usePermissionGrant("writeHttpMonitors");

const repo = await useHttpMonitorRepository();
const localePath = useLocalePath();

const onSubmit = async (data: HttpMonitorFormData) => {
  await repo.createHttpMonitor({
    ...data,
    ...data.notificationSettings,
    isActive: true
  });
  navigateTo(localePath("/dashboard/httpMonitors"));
};
</script>
<template>
  <BContainer>
    <BBreadcrumb>
      <BBreadcrumbItem :to="localePath('/dashboard')">{{
        $t("dashboard.mainSidebar.home")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem :to="localePath('/dashboard/httpMonitors')">{{
        $t("dashboard.mainSidebar.monitors")
      }}</BBreadcrumbItem>
      <BBreadcrumbItem active>
        {{ $t("dashboard.monitors.createMonitorTitle") }}
      </BBreadcrumbItem>
    </BBreadcrumb>
    <h2 class="mb-3">{{ $t("dashboard.monitors.createMonitorTitle") }}</h2>
    <HttpMonitorForm @submit="onSubmit" />
  </BContainer>
</template>
