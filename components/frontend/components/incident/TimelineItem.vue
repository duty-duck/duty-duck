<script setup lang="ts">
import type { TimelineItem } from 'bindings/TimelineItem';

const { item } = defineProps<{
    item?: TimelineItem
}>();
</script>

<template>
    <div class="event">
        <div class="dot-container">
            <UserAvatar :user="item.user" v-if="item?.user" size="2rem" font-size=".8rem" class="user-avatar"
                show-tooltip />
            <div v-else-if="item?.event.eventType === 'resolution'" class="big-dot bg-success">
                <Icon name="ph:check-bold" />
            </div>
            <div v-else-if="item?.event.eventType === 'confirmation'" class="big-dot bg-danger">
                <Icon name="ph:exclamation-mark-bold" />
            </div>
            <div class="dot" v-else></div>
        </div>
        <div class="event-inner" :class="{ 'has-content': $slots.default }">
            <slot />
            <template v-if="!$slots.default && item">
                <span class="text-secondary">{{ $d(new Date(item.event.createdAt), 'long') }}</span>
                <div>
                    <i18n-t v-if="item.event.eventType === 'creation'"
                        keypath="dashboard.incidents.timeline.incidentCreated" tag="span" />
                    <i18n-t v-else-if="item.event.eventType === 'confirmation'"
                        keypath="dashboard.incidents.timeline.incidentConfirmed" tag="span" />
                    <i18n-t v-if="item.event.eventType === 'notification'"
                        keypath="dashboard.incidents.timeline.notificationSent" tag="span" />
                    <i18n-t v-else-if="item.event.eventType === 'resolution'"
                        keypath="dashboard.incidents.timeline.incidentResolved" tag="span" />
                    <i18n-t v-else-if="item.event.eventType === 'monitorswitchedtorecovering'"
                        keypath="dashboard.incidents.timeline.monitorStatusSwitched" tag="span"
                        class="d-flex align-items-center gap-2">
                        <template #status>
                            <BBadge pill variant="info" class="text-white">{{ $t("dashboard.monitorStatus.recovering") }}</BBadge>
                        </template>
                    </i18n-t>
                    <i18n-t v-else-if="item.event.eventType === 'monitorswitchedtosuspicious'"
                        keypath="dashboard.incidents.timeline.monitorStatusSwitched" tag="span"
                        class="d-flex align-items-center gap-2">
                        <template #status>
                            <BBadge pill variant="warning">{{ $t("dashboard.monitorStatus.suspicious") }}</BBadge>
                        </template>
                    </i18n-t>
                    <i18n-t v-else-if="item.event.eventType === 'monitorswitchedtodown'"
                        keypath="dashboard.incidents.timeline.monitorStatusSwitched" tag="span"
                        class="d-flex align-items-center gap-2">
                        <template #status>
                            <BBadge pill variant="danger">{{ $t("dashboard.monitorStatus.down") }}</BBadge>
                        </template>
                    </i18n-t>
                    <i18n-t v-else-if="item.event.eventType === 'acknowledged'"
                        keypath="dashboard.incidents.timeline.incidentAcknowledged" tag="span">
                        <template #firstName>{{ item.user?.firstName }}</template>
                        <template #lastName>{{ item.user?.lastName }}</template>
                    </i18n-t>
                    <div v-else-if="item.event.eventType === 'comment' && item.event.eventPayload">
                        <div class="mb-2">{{ $t("dashboard.incidents.timeline.comment", {
                            firstName: item.user?.firstName, lastName:
                                item.user?.lastName
                        }) }}</div>
                        <DashboardCommentViewer :comment="(item.event.eventPayload as any).Comment" />
                    </div>
                    <div v-else-if="item.event.eventType === 'monitorpinged' && item.event.eventPayload">
                        <IncidentPingEvent :event="(item.event.eventPayload as any).MonitorPing" />
                    </div>
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

.big-dot {
    width: 1.25rem;
    height: 1.25rem;
    font-size: .8rem;
    border-radius: 50%;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    outline: 2px solid white;
    box-shadow: 1px 1px 4px rgba(0, 0, 0, 0.5);
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