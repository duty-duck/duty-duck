<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual';
import type { LogEvent } from './ViewerEvent.vue';

const viewerRef = ref<HTMLElement | null>(null);

const props = defineProps<{
    events: LogEvent[],
    bodyColumns: string[],
    canLoadNextEvents: boolean,
}>();

const emits = defineEmits<{
    loadNextEvents: []
}>();

const expandedEvent = ref(null as null | number);

// The virtualizer
const rowVirtualizer = useVirtualizer({
    count: 10000,
    getScrollElement: () => viewerRef.value,
    estimateSize: () => 35,
})

const eventsWithSize = computed(() => {
    const lineHeight = 36;
    const expandedSectionSize = 240;
    const menuSize = 32;

    return props.events.map((e, index) => ({
        data: e,
        index,
        size: (e as any).index === expandedEvent.value ? (lineHeight + expandedSectionSize + menuSize) : lineHeight
    }))
});

const onScrollEnd = () => {
    if (props.canLoadNextEvents) {
        emits('loadNextEvents')
    }
}

onMounted(() => {
    onScrollEnd()
})
</script>

<template>
    <div ref="viewerRef" class="viewer" :items="eventsWithSize" key-field="index" size-field="size"
        @scroll-end="onScrollEnd">

        <template v-slot="{ item: { data: event } }" v-for="event in rowVirtualizer.virtualItems">
            <LogViewerEvent :event="event" :body-columns="bodyColumns"
                @toggle="expandedEvent == event.index ? expandedEvent = null : expandedEvent = event.index"
                :expanded="expandedEvent == event.index" />
        </template>
    </div>
</template>

<style lang="scss" scoped>
.viewer {
    background-color: white;
    padding: .5rem;
    overflow: auto;
}
</style>
