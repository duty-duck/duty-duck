<script lang="ts" setup>
import { useIntervalFn } from "@vueuse/core";

const route = useRoute();
const router = useRouter();
const pageNumber = route.query.page ? Number(route.query.page) : 1;
const repository = useHttpMonitorRepository();

const { status, data, refresh } = await repository.useHttpMonitors({
  pageNumber,
  itemsPerPage: 10,
});
const numberOfPages = computed(() => data ? Number(data.totalNumberOfResults) / 10 : 1);

if (data.value?.items.length == 0 && pageNumber > 1) {
  router.replace("/dasboard/monitors");
}

useIntervalFn(() => {
  refresh();
}, 10000);
</script>

<template>
  <div>
    <BBreadcrumb>
      <BBreadcrumbItem to="/dashboard"> Home </BBreadcrumbItem>
      <BBreadcrumbItem active>Monitors</BBreadcrumbItem>
    </BBreadcrumb>
    <div class="d-flex align-items-center justify-content-between mb-3">
      <h2>Monitors</h2>
      <AddHttpMonitorButton />
    </div>
    <BAlert variant="danger" :model-value="status == 'error'">
      Failed to fetch HTTP monitors from the server. Please try again.
    </BAlert>
    <div
      v-if="data?.total_number_of_results == 0n"
      class="text-secondary text-center my-5"
    >
      <Icon name="ph:pulse-duotone" size="120px" />
      <h3>Nothing here yet</h3>
      <p class="lead">
        Create your first monitor to start monitoring your website
      </p>
      <AddHttpMonitorButton class="m-3" />
    </div>
    <MonitorCard v-for="monitor in data?.items" :key="monitor.id" v-bind="monitor" />
  </div>
</template>
