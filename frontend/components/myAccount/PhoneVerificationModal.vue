<script lang="ts" setup>
    const showModal = defineModel<boolean>();
    const code = ref<string>("");
    const state = ref<"initial" | "sending" | "sent" | "send-error" | "confirming" | "confirm-error">("initial");
    const userRepo = useUserRepository();
    const auth = useAuthMandatory();

    const codeIsValid = computed(() => code.value.length === 6);

    const sendCode = async () => {
        code.value = "";
        state.value = "sending";
        try {
            await userRepo.sendPhoneNumberVerificationCode();
            state.value = "sent";
        } catch (error) {
            console.error(error);
            state.value = "send-error";
        }
    }

    const confirmCode = async () => {
        state.value = "confirming";
        try {
            await userRepo.verifyPhoneNumber(code.value);
            await userRepo.refreshUserProfile();
            showModal.value = false;
        } catch (error) {
            console.error(error);
            state.value = "confirm-error";
        }
    }

</script>

<template>
    <BModal v-model="showModal" :title="$t('dashboard.myAccount.phoneVerificationModal.title')" :ok-disabled="!codeIsValid" @ok.prevent="confirmCode" @cancel="state = 'initial'">
        <template v-if="state === 'initial'">
            <p>
                {{ $t('dashboard.myAccount.phoneVerificationModal.initialInstructions') }}
            </p>
            <BButton @click="sendCode">{{ $t('dashboard.myAccount.phoneVerificationModal.sendCodeButton') }}</BButton>
        </template>

        <template v-else-if="state === 'sent'">
            <p>
                {{ $t('dashboard.myAccount.phoneVerificationModal.codeSentInstructions', { phoneNumber: auth.userProfile.user.phoneNumber }) }}
            </p>
            <BInput v-model="code" type="number" :placeholder="$t('dashboard.myAccount.phoneVerificationModal.codeInputPlaceholder')" maxlength="6" />
        </template>

        <template v-else-if="state === 'send-error'">
            <p>
                {{ $t('dashboard.myAccount.phoneVerificationModal.sendErrorMessage') }}
            </p>
            <BButton @click="sendCode">{{ $t('dashboard.myAccount.phoneVerificationModal.sendCodeButton') }}</BButton>
        </template>

        <template v-else-if="state === 'confirm-error'">
            <p>
                {{ $t('dashboard.myAccount.phoneVerificationModal.confirmErrorMessage') }}
            </p>
            <p>
                {{ $t('dashboard.myAccount.phoneVerificationModal.requestNewCodeInstructions') }}
            </p>
            <BButton @click="sendCode">{{ $t('dashboard.myAccount.phoneVerificationModal.sendCodeButton') }}</BButton>
        </template>

        <!-- Loading states -->
        <template v-else-if="state === 'sending' ">
            <BSpinner />
            {{ $t('dashboard.myAccount.phoneVerificationModal.sendingCodeMessage') }}
        </template>
        <template v-else-if="state === 'confirming' ">
            <BSpinner />
            {{ $t('dashboard.myAccount.phoneVerificationModal.verifyingCodeMessage') }}
        </template>
    </BModal>
</template>
