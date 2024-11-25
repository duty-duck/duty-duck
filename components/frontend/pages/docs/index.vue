<script setup lang="ts">
const { locale } = useI18n()
const { data } = await useAsyncData('docs-home', () => queryContent('docs', locale.value).find())
const localePath = useLocalePath()

onBeforeMount(() => {
    const path = data.value![0]!._path!.replace(`/${locale.value}/`, '/')
    navigateTo(localePath(path))
})
</script>

<template>
    <ShowcaseLayout></ShowcaseLayout>
</template>
