<script setup lang="ts">
const { incidentId } = defineProps<{
    incidentId: string
}>();

const repo = useIncidentRepository();
const { data, status, refresh } = await repo.useIncidentTimeline(incidentId);
defineExpose({
    refresh
});
</script>

<template>
    <section>
        <h5 class="mb-5">{{ $t("dashboard.incidents.timeline.sectionTitle") }}</h5>
        <IncidentTimelineItem v-for="item in data!.items" :key="`${item.event.incidentId}-${item.event.createdAt}`"
            :item="item" />
    </section>
</template>