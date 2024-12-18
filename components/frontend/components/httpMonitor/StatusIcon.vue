<script setup lang="ts">
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";

const props = defineProps<{
    status: HttpMonitorStatus;
    animated?: boolean,
    big?: boolean
}>();

const icon = computed(() => {
    if (props.status == "unknown" || props.status == "recovering") {
        return "ph:circle-duotone";
    }
    if (props.status == "down" || props.status == "suspicious") {
        return "ph:warning-circle-duotone";
    }
    if (props.status == "inactive") {
        return "ph:pause-circle-duotone"
    }

    return "ph:check-circle-duotone"
});
</script>

<template>
    <span class="monitor-icon" :class="{
        'animated': !!props.animated,
        'text-danger': props.status == 'down',
        'text-warning': props.status == 'suspicious',
        'text-info': props.status == 'recovering',
        'text-secondary': props.status == 'unknown' || props.status == 'inactive',
        'text-success': props.status == 'up',
        'text-secondary': props.status == 'archived'
    }">
        <template v-if="status != 'archived'">
            <Icon name="ph:circle-fill" :size="big ? '6rem' : '4rem'" class="secondary" v-show="animated" />
            <Icon name="ph:circle-fill" :size="big ? '6rem' : '4rem'" class="tertiary" v-show="animated" />
            <Icon :name="icon" :size="big ? '3rem' : '2rem'" />
        </template>
        <Icon v-else name="ph:archive-fill" :size="big ? '3rem' : '2rem'" />
    </span>

</template>

<style scoped lang="scss">
.monitor-icon {
    position: relative;

    @keyframes growsecondary {
        0% {
            opacity: 0;
            font-size: 1.7rem;
        }

        100% {
            opacity: .2;
            opacity: 3.8rem;
        }
    }

    @keyframes growtertiary {
        0% {
            opacity: 0;
            font-size: 1.7rem
        }

        100% {
            opacity: .1;
            opacity: 2rem;
        }
    }

    >.secondary {
        opacity: 0;
        animation: growsecondary 1.5s ease-in-out 0s infinite alternate;
    }

    >.tertiary {
        opacity: 0;
        animation: growtertiary 1.5s ease-in-out .5s infinite alternate;
    }

    >span {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }
}
</style>