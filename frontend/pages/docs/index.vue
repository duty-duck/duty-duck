<script setup lang="ts">
const { locale } = useI18n()
const { data } = await useAsyncData('docs-home', () => queryContent( 'docs', locale.value).find())

onMounted(() => {
    console.log(data?.value)
    navigateTo(data?.value?.[0]?._path)
})
</script>
