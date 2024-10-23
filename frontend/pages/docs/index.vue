<script setup lang="ts">
const { locale } = useI18n()
const { data } = await useAsyncData('docs-home', () => queryContent( 'docs', locale.value).find())

onBeforeMount(() => {
    navigateTo(data?.value?.[0]?._path)
})
</script>

<template>
    <ShowcaseLayout></ShowcaseLayout>
</template>
