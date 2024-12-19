<template>
    <div class="feature-block p-4 bg-light rounded-4" :class="`feature-block-${size}`">
        <slot name="start" />
        <div class="icon-wrapper rounded-3 mb-4 d-flex align-items-center justify-content-center"
            :class="iconBgColor || 'bg-primary bg-opacity-10'">
            <Icon class="icon" :class="[iconColor || 'text-primary']" :name="icon" />
        </div>
        <h3 class="fs-4 fw-semibold mb-3">{{ title }}</h3>
        <p class="text-secondary mb-0">{{ description }}</p>
        <slot name="end" />
    </div>
</template>

<script lang="ts" setup>
interface Props {
    title: string;
    description: string;
    icon: string;
    size: 'small' | 'medium' | 'large';
    iconBgColor?: string;
    iconColor?: string;
}

defineProps<Props>();
</script>

<style scoped lang="scss">
@import "~/assets/main.scss";

.feature-block {
    min-height: 200px;
    background: linear-gradient(315deg, rgba($primary, 0.1) 0%, white 40%);
    box-shadow: 0 0 10px rgba(0,0,0,0.05);
    width: 100%;
}

@include media-breakpoint-up(md) {
    .feature-block-small {
        width: 300px;
    }

    .feature-block-medium {
        width: 400px;
    }

    .feature-block-large {
        width: 500px;
    }
}

.icon-wrapper {
    width: 3rem;
    height: 3rem;
}

.icon {
    font-size: 1.5rem;
    line-height: 1;
}
</style>