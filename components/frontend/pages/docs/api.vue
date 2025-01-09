<template>
    <ShowcaseDocumentationLayout>
        <iframe class="mt-2" :src="`${serverUrl}/openapi/redoc`" />
    </ShowcaseDocumentationLayout>
</template>

<script setup lang="ts">
const { public: { serverUrl } } = useRuntimeConfig();
</script>

<style scoped>
iframe {
    width: 100%;
    height: 100vh;
}
</style>
