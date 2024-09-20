<script setup lang="ts">
import type { OrganizationUserRole } from 'bindings/OrganizationUserRole';
import type { ListOrganizationMembersItem } from 'bindings/ListOrganizationMembersItem';

const { t } = useI18n();
const repository = useOrganizationRepository();
const user = defineModel<ListOrganizationMembersItem | null>();
const emit = defineEmits<{
    (e: 'ok'): void
}>();

const options: { text: string, value: OrganizationUserRole, disabled?: boolean }[] = [
    { text: t('dashboard.organizationUserRoles.administrator'), value: 'Administrator' },
    { text: t('dashboard.organizationUserRoles.editor'), value: 'Editor' },
    { text: t('dashboard.organizationUserRoles.memberManager'), value: 'MemberManager' },
    { text: t('dashboard.organizationUserRoles.memberInviter'), value: 'MemberInviter' },
    { text: t('dashboard.organizationUserRoles.reporter'), value: 'Reporter' },
];
const selectedRoles = ref<OrganizationUserRole[]>([]);
const isLoading = ref(false);
watch(user, (newUser) => {
    selectedRoles.value = newUser?.organizationRoles ?? [];
});

const onSubmit = async () => {
    isLoading.value = true;
    try {
        await repository.changeMemberRole(user.value?.id!, { roles: selectedRoles.value })
        emit('ok')
    } finally {
        isLoading.value = false;
    }
}
</script>


<template>
    <BModal :title="$t('dashboard.myOrg.updateUserRolesModal.title')" :model-value="!!user"
        @update:model-value="user = null" @ok="onSubmit" :disabled="isLoading"
        :ok-title="isLoading ? $t('dashboard.myOrg.updateUserRolesModal.loading') : $t('dashboard.myOrg.updateUserRolesModal.ok')">
        {{ $t('dashboard.myOrg.updateUserRolesModal.description', {
            firstName: user?.firstName, lastName: user?.lastName
        }) }}
        <BFormCheckboxGroup v-model="selectedRoles" :options="options" class="my-3" switches size="lg" stacked />
    </BModal>
</template>