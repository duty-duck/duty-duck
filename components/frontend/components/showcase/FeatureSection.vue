<template>
    <section class="container">
        <!-- Section header -->
        <div class="mb-3">
            <h2 class="display-5 fw-semibold mb-4">{{ title }}</h2>
            <p class="lead text-secondary col-md-6">{{ description }}</p>
        </div>

        <!-- Feature blocks -->
        <div class="d-flex gap-2 gap-lg-3 flex-wrap" :class="blockJustifyContentClass">
            <slot></slot>
        </div>
    </section>
</template>

<script lang="ts" setup>
interface Props {
    title: string;
    description: string;
    blockJustifyContent?: "center" | "start" | "end";
}

const props = defineProps<Props>();

const blockJustifyContentClass = computed(() => {
    return props.blockJustifyContent ? `justify-content-${props.blockJustifyContent}` : "";
});
</script>

<style scoped lang="scss">
@import "~/assets/main.scss";

section {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 3rem 1rem;

    @include media-breakpoint-up(md) {
        padding: 5rem 1rem;
    }

    @include media-breakpoint-up(lg) {
        padding: 8rem 1rem;
    }

    @include media-breakpoint-up(xl) {
    }
}
</style>