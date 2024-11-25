<template>
    <p v-html="data.text" />
</template>

<script setup lang="ts">
const { data } = defineProps<{
    data: { text: string }
}>();
</script>

<style scoped>
p:last-of-type {
    margin-bottom: 0;
}
</style>