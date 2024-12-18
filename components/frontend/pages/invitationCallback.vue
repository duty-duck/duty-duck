<script lang="ts" setup>
import useVuelidate from "@vuelidate/core";
import { required, sameAs } from "@vuelidate/validators";
import { useRoute } from "vue-router";
import { type AcceptInvitationCommand } from "bindings/AcceptInvitationCommand";

const localePath = useLocalePath();
const keycloak = await useKeycloak();
const repo = await usePublicInvitationRepository();

const { organizationId, invitationId } = useRoute().query;
const { data: invitation } = await repo.useInvitation(organizationId as string, invitationId as string);
const isStrongPassword = usePasswordValidator(
    computed(() => [formData.firstName, formData.lastName])
);

const formData = reactive({
    firstName: "",
    lastName: "",
    password: "",
    passwordConfirm: "",
});

const formRules = {
    firstName: { required },
    lastName: { required },
    password: { required, isStrongPassword },
    passwordConfirm: {
        required,
        sameAsPassword: sameAs(computed(() => formData.password)),
    },
};

const passwordFieldState = computed(() => {
    if (!v$.value.password.$dirty || v$.value.password.$pending) {
        return null;
    }
    return !v$.value.password.$invalid;
});

const invitationState = ref<'accepted' | 'rejected' | 'error' | 'pending'>('pending');

const v$ = useVuelidate(formRules, formData);

const onAccept = async () => {
    if (!invitation.value?.invitee && v$.value.$invalid) {
        return;
    }

    const command: AcceptInvitationCommand = {
        userDetails: invitation.value?.invitee ? null : {
            firstName: formData.firstName,
            lastName: formData.lastName,
            password: formData.password,
        }
    };

    try {
        await repo.acceptInvitation(organizationId as string, invitationId as string, command);
        invitationState.value = 'accepted';
    } catch (error) {
        invitationState.value = 'error';
    }
}

const onReject = async () => {
    try {
        await repo.rejectInvitation(organizationId as string, invitationId as string);
        invitationState.value = 'rejected';
    } catch (error) {
        invitationState.value = 'error';
    }
}

onMounted(async () => {
    if (keycloak.keycloakState.value) {
        await keycloak.logout();
    }
});
</script>

<template>
    <div id="container">
        <BCard :title="`Join organization ${invitation?.organization.displayName}`" autofill="false"
            v-if="invitationState === 'pending'">
            <p>
                You have been invited to join the organization {{ invitation?.organization.displayName }} by {{
                    invitation?.inviter.firstName }} {{ invitation?.inviter.lastName }}.
            </p>
            <BForm v-if="invitation?.invitee == null" class="mb-4">
                <p>We need you to fill in the form below to complete the registration process.</p>
                <BFormGroup label="Email" class="mb-3" id="emailGroup"
                    description="We won't share your e-mail with anyone. The e-mail address is bound to this invitation and cannot be edited.">
                    <input class="form-control" :value="invitation?.invitation.email" disabled />
                </BFormGroup>
                <BFormGroup label="First name" :invalid-feedback="v$.firstName.$errors[0]?.$message.toString()"
                    id="firstNameGroup" label-for="firstNameInput" class="mb-3" required>
                    <BFormInput v-model="v$.firstName.$model" id="firstNameInput"
                        placeholder="Enter your first name please"
                        :state="v$.firstName.$dirty ? !v$.firstName.$invalid : null" />
                </BFormGroup>
                <BFormGroup label="Last name" :invalid-feedback="v$.lastName.$errors[0]?.$message.toString()"
                    label-for="lastNameInput" class="mb-3" required id="lastNameGroup">
                    <BFormInput v-model="v$.lastName.$model" id="lastNameInput"
                        placeholder="Enter your last name please"
                        :state="v$.lastName.$dirty ? !v$.lastName.$invalid : null" />
                </BFormGroup>
                <div class="row mb-3">
                    <BFormGroup class="col-md-6" label="Password" id="passwordGroup"
                        :invalid-feedback="v$.password.$errors[0]?.$message.toString()"
                        description="We recommend using a strong, random password" label-for="passwordInput">
                        <BFormInput v-model="v$.password.$model" type="password" id="passwordInput"
                            placeholder="Enter your password please" :state="passwordFieldState" />
                    </BFormGroup>
                    <BFormGroup class="col-md-6" label="Password confirmation" id="passwordConfirmGroup"
                        :invalid-feedback="v$.passwordConfirm.$errors[0]?.$message.toString()"
                        placeholder="Enter your password again please" label-for="passwordConfirmInput">
                        <BFormInput v-model="v$.passwordConfirm.$model" type="password" id="passwordConfirmInput"
                            :state="v$.passwordConfirm.$dirty ? !v$.passwordConfirm.$invalid : null" />
                    </BFormGroup>
                </div>
            </BForm>
            <p>
                You can choose to accept or reject the invitation using the buttons below.
            </p>
            <div class="d-flex gap-3">
                <BButton class="flex-grow-1" variant="outline-secondary" pill @click="onReject">Reject</BButton>
                <BButton class="flex-grow-1" variant="success" pill :disabled="!invitation?.invitee && v$.$invalid"
                    @click="onAccept">
                    Accept</BButton>
            </div>
        </BCard>
        <BCard title="Invitation accepted" v-else-if="invitationState === 'accepted'">
            <p>You have accepted the invitation to join the organization {{ invitation?.organization.displayName }}.</p>
            <NuxtLink :to="localePath('/dashboard')">Go to dashboard</NuxtLink>
        </BCard>
        <BCard title="Invitation rejected" v-else-if="invitationState === 'rejected'">
            <p>You have rejected the invitation to join the organization {{ invitation?.organization.displayName }}.</p>
            <NuxtLink :to="localePath('/')">Go to home page</NuxtLink>
        </BCard>
        <BCard title="Invitation error" v-else>
            <p>An error occurred while processing your invitation. Please retry later or contact the organization
                administrator.</p>
        </BCard>
    </div>
</template>

<style scoped lang="scss">
#container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
}
</style>