<script lang="ts" setup>
const emit = defineEmits<{
    ok: []
}>();

const isModalVisible = ref(false);
</script>

<template>
    <BModal v-model="isModalVisible" :title="$t('dashboard.incidents.resolve.modal.title')" @ok="emit('ok')"
        :ok-variant="'danger'" :ok-title="$t('dashboard.incidents.resolve.modal.confirm')" :cancel-title="$t('cancel')">
        <p>{{ $t('dashboard.incidents.resolve.modal.message') }}</p>
    </BModal>
    <BButton size="sm" @click="isModalVisible = true" variant="outline-secondary"
        class="d-flex align-items-center gap-1">
        <Icon name="ph:x-square" />
        {{ $t('dashboard.incidents.resolve.buttonLabel') }}
    </BButton>
</template>