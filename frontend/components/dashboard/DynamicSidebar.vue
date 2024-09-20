<script setup lang="ts">
const route = useRoute();
const MainSidebar = resolveComponent('DashboardMainSidebar');
const SettingsSidebar = resolveComponent('DashboardSettingsSidebar');
const showSettingsSidebar = computed(() => route.path.includes('myOrg') || route.path.includes('myAccount'));
</script>

<template>
    <component :is="showSettingsSidebar ? SettingsSidebar : MainSidebar" />
</template>