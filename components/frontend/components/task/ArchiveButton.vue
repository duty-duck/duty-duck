<template>
  <div>
    <BButton class="icon-link" variant="outline-secondary" @click="showModal" :disabled="isArchiving">
      <span v-if="isArchiving" class="spinner-border spinner-border-sm me-1" />
      <Icon name="ph:archive-fill" />
      {{ $t('dashboard.tasks.archive.button') }}
    </BButton>

    <BModal v-model="isModalVisible" :title="$t('dashboard.tasks.archive.modal.title')" @ok="handleArchive"
      :ok-variant="'danger'" :ok-title="$t('dashboard.tasks.archive.modal.confirm')" :cancel-title="$t('cancel')"
      :ok-disabled="isArchiving">
      <p>{{ $t('dashboard.tasks.archive.modal.message') }}</p>
    </BModal>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  taskId: string
}>()

const emit = defineEmits<{
  archived: []
}>()

const isModalVisible = ref(false)
const isArchiving = ref(false)
const repository = useTasksRepository()

const showModal = () => {
  isModalVisible.value = true
}

const handleArchive = async () => {
  try {
    isArchiving.value = true
    await repository.archiveTask(props.taskId)
    isModalVisible.value = false
    emit('archived')
  } catch (error) {
    console.error('Failed to archive task:', error)
  } finally {
    isArchiving.value = false
  }
}
</script>
