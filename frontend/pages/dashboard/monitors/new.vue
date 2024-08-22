<script lang="ts" setup>
import type { HttpMonitorFormData } from "~/components/HttpMonitorForm.vue";

const repo = useHttpMonitorRepository();
const router = useRouter();

const onSubmit = async (data: HttpMonitorFormData) => {
  await repo.createHttpMonitor({
    ...data,
    isActive: true
  });
  router.push("/dashboard/monitors");
};
</script>
<template>
  <div>
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard"> Home </BBreadcrumbItem>
      <BBreadcrumbItem to="/dashboard/monitors">Monitors</BBreadcrumbItem>
      <BBreadcrumbItem active>New monitor</BBreadcrumbItem>
    </BBreadcrumb>
    <h2 class="mb-3">Create a new monitor</h2>
    <HttpMonitorForm @submit="onSubmit" />
  </div>
</template>
