// FeatureSection.vue
<template>
    <section class="w-100 py-5" style="touch-action: pan-y pinch-zoom;">
        <!-- Section header -->
        <div class="mb-5">
            <h2 class="display-4 fw-semibold mb-4">{{ title }}</h2>
            <p class="lead text-secondary col-md-8">{{ description }}</p>
        </div>

        <!-- Horizontal scrolling content -->
        <div class="feature-content d-flex gap-4 overflow-auto">
            <template v-for="(feature, index) in features" :key="index">
                <!-- Feature block -->
                <div class="feature-block bg-light rounded-4 p-4 flex-shrink-0" :class="[getSizeClass(feature.size)]">
                    <!-- Icon -->
                    <div class="icon-wrapper rounded-3 mb-4 d-flex align-items-center justify-content-center"
                        :class="feature.iconBgColor || 'bg-success bg-opacity-10'">
                        <Icon class="icon" :class="[feature.iconColor || 'text-success']" :name="feature.icon" />
                    </div>

                    <!-- Content -->
                    <h3 class="fs-4 fw-semibold mb-3">{{ feature.title }}</h3>
                    <p class="text-secondary mb-0">{{ feature.description }}</p>
                </div>
            </template>
        </div>
    </section>
</template>

<script lang="ts" setup>
interface Feature {
    title: string;
    description: string;
    icon: string;
    size: 'small' | 'medium' | 'large';
    iconBgColor?: string;
    iconColor?: string;
}

interface Props {
    title: string;
    description: string;
    features: Feature[];
}

const props = defineProps<Props>();


const getSizeClass = (size: string) => {
    switch (size) {
        case 'small':
            return 'feature-block-small';
        case 'medium':
            return 'feature-block-medium';
        case 'large':
            return 'feature-block-large';
        default:
            return 'feature-block-small';
    }
};

</script>

<style scoped lang="scss">
@import "~/assets/main.scss";

section {
    min-height: 75vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding-left: 1rem;

    @include media-breakpoint-up(md) {
        padding-left: 2rem;
    }

    @include media-breakpoint-up(lg) {
        padding-left: 4rem;
    }

    @include media-breakpoint-up(xl) {
        padding-left: 8rem;
    }
}

.feature-content {
    transition: transform 0.1s ease-out;
    will-change: transform;
}

.feature-block {
    min-height: 200px;
    background-color: green !important;
}

.feature-block-small {
    width: 300px;
}

.feature-block-medium {
    width: 400px;
}

.feature-block-large {
    width: 500px;
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