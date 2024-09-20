<script lang="ts" setup>
import type { HttpMonitorFormData } from "~/components/httpMonitor/Form.vue";

ensurePemissionOnBeforeMount("writeHttpMonitors");

const repo = useHttpMonitorRepository();
const router = useRouter();
const localePath = useLocalePath();

const onSubmit = async (data: HttpMonitorFormData) => {
  await repo.createHttpMonitor({
    ...data,
    isActive: true
  });
  router.push(localePath("/dashboard/httpMonitors"));
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
