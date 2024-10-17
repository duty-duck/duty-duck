<template>
    <h5 v-html="data.text" />
</template>

<script setup lang="ts">
const { data } = defineProps<{
    data: { text: string }
}>();
</script>