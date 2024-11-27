<script setup lang="ts">
const path = await useDocumentationFirstPage();

onBeforeMount(() => {
    navigateTo(path.value)
})
</script>

