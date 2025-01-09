<script setup lang="ts">
import { BAlert as BaseComponent } from 'bootstrap-vue-next';

const props = defineProps<{
    variant: 'info' | 'warning' | 'danger';
}>();
</script>

<template>
    <BaseComponent :model-value="true" :variant="props.variant">
        <ContentSlot unwrap="p" />
    </BaseComponent>
</template>
