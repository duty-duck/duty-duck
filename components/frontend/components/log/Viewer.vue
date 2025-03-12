<script setup lang="ts">
import { RecycleScroller } from 'vue-virtual-scroller';
import type { LogEvent } from './ViewerEvent.vue';

const props = defineProps<{
    events: LogEvent[],
    bodyColumns: string[],
    canLoadNextEvents: boolean,
}>();

const emits = defineEmits<{
    loadNextEvents: []
}>();

const expandedEvent = ref(null as null | number);

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
    <RecycleScroller class="viewer" :items="eventsWithSize" key-field="index" size-field="size"
        @scroll-end="onScrollEnd">
        <template v-slot="{ item: { data: event } }">
            <LogViewerEvent :event="event" :body-columns="bodyColumns"
                @toggle="expandedEvent == event.index ? expandedEvent = null : expandedEvent = event.index"
                :expanded="expandedEvent == event.index" />
        </template>
    </RecycleScroller>

</template>

<style lang="scss" scoped>
.viewer {
    background-color: white;
    padding: .5rem;
}
</style>