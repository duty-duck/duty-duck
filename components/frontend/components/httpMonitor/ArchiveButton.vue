<template>
  <div>
    <BButton
      class="icon-link"
      variant="outline-secondary"
      @click="showModal"
      :disabled="isArchiving"
    >
      <span v-if="isArchiving" class="spinner-border spinner-border-sm me-1" />
        <Icon name="ph:archive-fill" />
      {{ $t('dashboard.monitors.archive.button') }}
    </BButton>

    <BModal
      v-model="isModalVisible"
      :title="$t('dashboard.monitors.archive.modal.title')"
      @ok="handleArchive"
      :ok-variant="'danger'"
      :ok-title="$t('dashboard.monitors.archive.modal.confirm')"
      :cancel-title="$t('cancel')"
      :ok-disabled="isArchiving"
    >
      <p>{{ $t('dashboard.monitors.archive.modal.message') }}</p>
    </BModal>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useHttpMonitorRepository } from '~/composables/useHttpMonitorRepository'

const props = defineProps<{
  monitorId: string
}>()

const emit = defineEmits<{
  archived: []
}>()

const isModalVisible = ref(false)
const isArchiving = ref(false)
const repository = useHttpMonitorRepository()

const showModal = () => {
  isModalVisible.value = true
}

const handleArchive = async () => {
  try {
    isArchiving.value = true
    await repository.archiveHttpMonitor(props.monitorId)
    isModalVisible.value = false
    emit('archived')
  } catch (error) {
    console.error('Failed to archive monitor:', error)
  } finally {
    isArchiving.value = false
  }
}
</script>
