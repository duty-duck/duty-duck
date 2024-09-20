<script setup lang="ts">
import type { User } from 'bindings/User';

const user = defineModel<User | null>();
const emit = defineEmits<{
    (e: 'ok'): void
}>();

const isLoading = ref(false);
const organizationRepository = useOrganizationRepository();
const confirmationInput = ref('');

const onSubmit = async () => {
    isLoading.value = true;
    try {
        await organizationRepository.removeMember(user.value!.id);
        emit('ok');
    } catch (error) {
        console.error(error);
    } finally {
        isLoading.value = false;
        emit('ok');
    }
}
</script>

<template>
    <BModal :title="$t('dashboard.myOrg.revokeUserModal.title')" :model-value="!!user" @update:model-value="user = null" @ok="onSubmit"
        :ok-disabled="isLoading || confirmationInput !== `${user?.firstName} ${user?.lastName}`"
        :ok-title="isLoading ? $t('dashboard.myOrg.revokeUserModal.revoking') : $t('dashboard.myOrg.revokeUserModal.revokeUser')" ok-variant="danger">
        <p>{{ $t('dashboard.myOrg.revokeUserModal.confirmationMessage', { firstName: user?.firstName, lastName: user?.lastName }) }}</p>
        <p>{{ $t('dashboard.myOrg.revokeUserModal.instructionMessage') }}</p>
        <BFormInput v-model="confirmationInput"
            :placeholder="$t('dashboard.myOrg.revokeUserModal.inputPlaceholder', { firstName: user?.firstName, lastName: user?.lastName })" />
    </BModal>
</template>