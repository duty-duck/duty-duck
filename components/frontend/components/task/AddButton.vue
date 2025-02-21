<script setup lang="ts">
const localePath = useLocalePath();
const { userHasPermissionComputed } = await useAuth();
const canCreateTasks = userHasPermissionComputed('writeTasks');
</script>

<template>
  <BButton variant="primary" size="sm" :to="localePath('/dashboard/tasks/new')" v-if="canCreateTasks">
    <Icon name="ph:plus-bold" />

    {{ $t("dashboard.tasks.createTaskButton") }}
  </BButton>
</template>
