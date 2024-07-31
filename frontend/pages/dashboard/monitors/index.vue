<script lang="ts" setup>
import { useIntervalFn } from "@vueuse/core";

const route = useRoute();
const router = useRouter();
const pageNumber = computed(() => route.query.page ? Number(route.query.page) : 1);
const fetchParams = computed(() => ({
  pageNumber: pageNumber.value,
  itemsPerPage: 10,
}));

const repository = useHttpMonitorRepository();

const { status, data, refresh } = await repository.useHttpMonitors(fetchParams);

const onPageChange = (pageNumber: number) => {
  router.push(`/dashboard/monitors?page=${pageNumber}`)
}

if (data.value?.items.length == 0 && pageNumber.value > 1) {
  router.replace("/dasboard/monitors");
}

useIntervalFn(() => {
  refresh();
}, 10000);
</script>

<template>
  <div>
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard">Home</BBreadcrumbItem>
      <BBreadcrumbItem active>Monitors</BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center justify-content-between">
      <h2>Monitors</h2>
      <AddHttpMonitorButton />
    </div>
    <div class="small text-secondary mb-4">{{ data?.totalNumberOfResults }} Total Monitors, 10 items per page</div>
    <BAlert variant="danger" :model-value="status == 'error'">
      Failed to fetch HTTP monitors from the server. Please try again.
    </BAlert>
    <div v-if="data?.totalNumberOfResults == 0" class="text-secondary text-center my-5">
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>Nothing here yet</h3>
      <p class="lead">
        Create your first monitor to start monitoring your website
      </p>
      <AddHttpMonitorButton class="m-3" />
    </div>
    <MonitorCard v-for="monitor in data?.items" :key="monitor.id" v-bind="monitor" />
    <BPagination :model-value="pageNumber" @update:modelValue="onPageChange" :total-rows="data?.totalNumberOfResults"
      :per-page="10" prev-text="Prev" next-text="Next" />
  </div>
</template>
