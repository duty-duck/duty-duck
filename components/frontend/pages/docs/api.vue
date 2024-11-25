<template>
    <ShowcaseLayout>
        <div class="bg-gray-100 px-4 pt-2">
            <BBreadcrumb>
                <BBreadcrumbItem>
                    <NuxtLink to="/">
                        Home
                    </NuxtLink>
                </BBreadcrumbItem>
                <BBreadcrumbItem>
                    <NuxtLink to="/docs">Documentation</NuxtLink>
                </BBreadcrumbItem>
                <BBreadcrumbItem>
                    API Docs
                </BBreadcrumbItem>
            </BBreadcrumb>
        </div>
        <iframe :src="`${serverUrl}/redoc`" />
    </ShowcaseLayout>
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
