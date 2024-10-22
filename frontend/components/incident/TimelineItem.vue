<script setup lang="ts">
import type { TimelineItem } from 'bindings/TimelineItem';

const { item } = defineProps<{
    item?: TimelineItem
}>();
</script>

<template>
    <div class="event">
        <div class="dot-container">
            <UserAvatar :user="item.user" v-if="item?.user" size="2rem" font-size=".8rem" class="user-avatar" show-tooltip />
            <div class="dot" v-else></div>
        </div>
        <div class="event-inner" :class="{ 'has-content': $slots.default }">
            <slot />
            <template v-if="!$slots.default && item">
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
                                item.user?.lastName
                        }) }}
                    </span>
                    <DashboardCommentViewer 
                        class="mb-3 mt-2"
                        v-else-if="item.event.eventType === 'comment' && item.event.eventPayload" 
                        :comment="(item.event.eventPayload as any).Comment" 
                    />
                </div>
            </template>
        </div>
    </div>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";
$dot-size: 0.5rem;
@include media-breakpoint-up(md) {
    $dot-size: 0.75rem;
}

.event {
    margin-left: .25rem;
    border-left: 2px solid var(--bs-gray-300);
    position: relative;

    @include media-breakpoint-up(md) {
        margin-left: 2rem;
    }
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
    padding-left: 1.5rem;
    padding-bottom: 1rem;
    top: -0.85rem;


    @include media-breakpoint-up(md) {
        padding-left: 2rem;
        top: -0.75rem;
    }
}
</style>