<template>
    <BButton
      class="icon-link"
      variant="outline-secondary"
      @click="handleToggle"
      :disabled="isToggling"
    >
      <template v-if="status === 'inactive'">
        <Icon name="ph:play-fill" />
        {{ $t("dashboard.monitors.start") }}
      </template>
      <template v-else>
        <Icon name="ph:pause-fill" />
        {{ $t("dashboard.monitors.pause") }}
      </template>
    </BButton>
  </template>
  
  <script setup lang="ts">
  import { ref } from 'vue'
  import { useHttpMonitorRepository } from '~/composables/useHttpMonitorRepository'
  
  const props = defineProps<{
    monitorId: string
    status: string
  }>()
  
  const emit = defineEmits<{
    toggled: []
  }>()
  
  const isToggling = ref(false)
  const repository = useHttpMonitorRepository()
  
  const handleToggle = async () => {
    try {
      isToggling.value = true
      await repository.toggleHttpMonitor(props.monitorId)
      emit('toggled')
    } catch (error) {
      console.error('Failed to toggle monitor:', error)
    } finally {
      isToggling.value = false
    }
  }
  </script>