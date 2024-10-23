<script setup lang="ts">
import useVuelidate from '@vuelidate/core';
import { email, minLength, required } from '@vuelidate/validators';
import type { OrganizationUserRole } from 'bindings/OrganizationUserRole';
import StatusLabel from '../httpMonitor/StatusLabel.vue';

const showModal = defineModel<boolean>();
const emit = defineEmits<{
    (e: 'ok'): void
}>();

const isLoading = ref<boolean>(false);
const repo = await useOrganizationRepository();
const { t } = useI18n();
const { show } = useToast();
const { userProfile: { active_organization, organization_roles } } = await useAuth();

const roleOptions: { text: string, value: OrganizationUserRole, disabled?: boolean }[] = [
    { text: t('dashboard.organizationUserRoles.administrator'), value: 'Administrator', disabled: !organization_roles.includes('Administrator') },
    { text: t('dashboard.organizationUserRoles.editor'), value: 'Editor' },
    { text: t('dashboard.organizationUserRoles.memberManager'), value: 'MemberManager' },
    { text: t('dashboard.organizationUserRoles.memberInviter'), value: 'MemberInviter' },
    { text: t('dashboard.organizationUserRoles.reporter'), value: 'Reporter' },
];

const formState = reactive({
    email: '',
    role: null as OrganizationUserRole | null
});

const rules = {
    email: { required, email },
    role: { required },
};

const v$ = useVuelidate(rules, formState);

const onSubmit = async () => {
    isLoading.value = true;
    try {
        await repo.inviteMember({
            email: formState.email,
            role: formState.role!
        });

        show?.({
            props: {
                title: t('dashboard.myOrg.inviteUserModal.successToastNotification.title'),
                body: t('dashboard.myOrg.inviteUserModal.successToastNotification.body'),
                variant: 'success',
                value: 5000
            }
        });

        emit('ok');
    } catch (error) {
        console.error(error);
        show?.({
            props: {
                title: t('dashboard.myOrg.inviteUserModal.errorToastNotification.title'),
                body: t('dashboard.myOrg.inviteUserModal.errorToastNotification.body'),
                variant: 'danger',
                value: 5000
            }
        });
    } finally {
        isLoading.value = false;
        showModal.value = false;
    }
};
</script>

<template>
    <BModal :title="$t('dashboard.myOrg.inviteUserModal.title')" v-model="showModal"
        :ok-title="isLoading ? $t('dashboard.myOrg.inviteUserModal.loading') : $t('dashboard.myOrg.inviteUserModal.invite')"
        @ok="onSubmit" :ok-disabled="v$.$invalid || isLoading">
        <p>
            {{ $t('dashboard.myOrg.inviteUserModal.description', { organization: active_organization.displayName }) }}
        </p>
        <p>{{ $t('dashboard.myOrg.inviteUserModal.emailDescription') }}</p>
        <BFormGroup :label="$t('dashboard.myOrg.inviteUserModal.emailLabel')" class="mb-3">
            <BFormInput v-model="v$.email.$model" type="email"
                :placeholder="$t('dashboard.myOrg.inviteUserModal.emailPlaceholder')" />
        </BFormGroup>
        <BFormGroup :label="$t('dashboard.myOrg.inviteUserModal.roleLabel')">
            <BFormSelect v-model="v$.role.$model" :options="roleOptions" class="my-3" />
        </BFormGroup>
    </BModal>
</template>