<script lang="ts" setup>
const localePath = useLocalePath();
const { taskId } = defineProps<{
    taskId: string,
}>();

const repo = useTasksRepository();
const { data: taskRes } = await repo.useTask(taskId);
</script>

<template>
    <div class="details-container">
        <NuxtLink :to="localePath(`/dashboard/tasks/${taskRes?.task.id}`)" class="icon-link" v-b-tooltip.hover.top
            :title="$t('dashboard.incidents.goToSource')">
            <Icon name="ph:pulse-duotone" size="22px" />
            {{ $t('dashboard.tasks.scheduledTask') }}
        </NuxtLink>
        <h2> {{ taskRes?.task.name }} </h2>
    </div>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.details-container {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;

    h2 {
        font-size: 1rem;
        font-weight: normal;
    }
}
</style>