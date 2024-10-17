<script setup lang="ts">
import type { TimelineItem } from 'bindings/TimelineItem';

const { item } = defineProps<{
    item: TimelineItem
}>();
</script>

<template>
    <div class="event">
        <div class="dot-container">
            <UserAvatar :user="item.user" v-if="item.user" size="2rem" font-size=".8rem" class="user-avatar" />
            <div class="dot" v-else></div>
        </div>
        <div class="event-inner">
            <span class="text-secondary">{{ $d(new Date(item.event.createdAt), 'long') }}</span>
            <div>
                <span v-if="item.event.eventType === 'creation'">
                    {{ $t("dashboard.incidents.timeline.incidentCreated") }}
                </span>
                <span v-if="item.event.eventType === 'notification'">
                    {{ $t("dashboard.incidents.timeline.notificationSent") }}
                </span>
                <span v-else-if="item.event.eventType === 'resolution'">
                    {{ $t("dashboard.incidents.timeline.incidentResolved") }}
                </span>
                <span v-else-if="item.event.eventType === 'acknowledged'">
                    {{ $t("dashboard.incidents.timeline.incidentAcknowledged", {
                        firstName: item.user?.firstName, lastName:
                            item.user?.lastName }) }}
                </span>
            </div>

        </div>
    </div>
</template>

<style scoped lang="scss">
$dot-size: 0.75rem;

.event {
    margin-left: 2rem;
    border-left: 2px solid var(--bs-gray-300);
    position: relative;
}

.dot-container {
    position: absolute;
    top: 0;
    transform: translateY(-50%) translateX(-50%);
}

.user-avatar {
    position: relative;
    top: .1rem;
}

.dot {
    width: $dot-size;
    height: $dot-size;
    border-radius: 50%;
    background-color: var(--bs-gray-500);
}

.event-inner {
    position: relative;
    padding-left: 2rem;
    padding-bottom: .5rem;
    top: -$dot-size;
}
</style>