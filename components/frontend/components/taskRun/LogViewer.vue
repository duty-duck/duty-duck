<!-- Displays the logs of a task run (or a message if there is no log) -->
<!-- Wraps the LogViewer component, which is a purely presentational component, with the logic to fetch the logs from the API -->

<script setup lang="ts">
import type { LogEvent } from '../log/ViewerEvent.vue';
import { LogViewer } from '#components';

const { taskId, taskRunId, taskRunIsActive } = defineProps<{
    taskId: string,
    taskRunId: string,
    taskRunIsActive: boolean
}>();

const repo = useTasksRepository();

const state = reactive({
    totalNumberOfEvents: 0,
    status: 'initial' as 'loading' | 'error' | 'loaded' | 'initial',
    loadedEvents: [] as LogEvent[],
});

const lastLoadedEventIndex = computed(() => {
    if (state.loadedEvents.length > 0) {
        const event = state.loadedEvents[state.loadedEvents.length - 1];
        if (event.eventType == 'taskRun') {
            return event.index
        }

        return null;
    }

    return null;
});

/**
 * Returns whether there are next event available to load from the server
 */
const canLoadNextEvents = computed(() => {
    if (state.status == 'initial') {
        return true
    }
    if (state.status == 'loaded') {
        if (state.totalNumberOfEvents > (lastLoadedEventIndex.value! + 1)) {
            return true
        }
    }

    return false
});

const loadNextEvents = async () => {
    state.status = 'loading';

    try {
        const { logs, totalNumberOfLogs } = await repo.fetchTaskRunLogs(taskId, taskRunId, lastLoadedEventIndex.value ? lastLoadedEventIndex.value + 1 : 0);
        state.totalNumberOfEvents = totalNumberOfLogs as any as number;
        state.loadedEvents = [...state.loadedEvents, ...logs.map(l => {
            const event: LogEvent = { eventType: 'taskRun', ...l };
            return event
        })];

        state.status = 'loaded';
    } catch (e) {
        console.error(e);
        state.status = 'error';
    }
}

// setup periodic refresh
useDataRefreshInterval(() => {
    if (taskRunIsActive) {
        loadNextEvents();
    }
})
</script>

<template>
    <template v-if="state.status == 'error'">
        <BCard class="text-center py-5">
            <Icon name="ph:warning-duotone" size="4rem" class="m" />
            <h3>{{ $t('dashboard.taskRuns.logs.error.title') }}</h3>
            <p>{{ $t('dashboard.taskRuns.logs.error.description') }}</p>
            <BButton @click="state.status = 'initial'">
                {{ $t('retry') }}
            </BButton>
        </BCard>
    </template>
    <template v-else-if="!taskRunIsActive && state.status == 'loaded' && state.totalNumberOfEvents == 0">
        <BCard class="text-center py-5">
            <Icon name="ph:list-magnifying-glass" size="4rem" />
            <h3>{{ $t('dashboard.taskRuns.logs.noLogs.title') }}</h3>
            <p>{{ $t('dashboard.taskRuns.logs.noLogs.description') }}</p>
        </BCard>
    </template>
    <template v-else>
        <BCard body-class="py-1 px-0">
            <LogViewer :events="state.loadedEvents" :body-columns="['message']"
                :can-load-next-events="canLoadNextEvents" @load-next-events="loadNextEvents"
                style="height: 700px; max-height: 70vh;" />
        </BCard>
    </template>

</template>