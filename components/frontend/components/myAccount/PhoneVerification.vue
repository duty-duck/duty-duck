<script setup lang="ts">
const auth = await useAuth();
const phoneVerificationModal = ref<boolean>(false);

defineEmits(['refresh']);
</script>

<template>
  <BCard>
    <BCardTitle class="d-flex align-items-center gap-2">
      {{ $t('dashboard.myAccount.phoneNumberVerification') }}
      <Icon name="ph:check-circle-fill" class="text-success" v-if="auth.userProfile.user.phoneNumberVerified" />
      <Icon name="ph:x-circle-fill" class="text-danger" v-else />
    </BCardTitle>
    <p v-if="!auth.userProfile.user.phoneNumber">
      {{ $t("dashboard.myAccount.noPhoneNumberConfigured") }}
    </p>
    <template v-else-if="!auth.userProfile.user.phoneNumberVerified">
      <p>
        {{ $t("dashboard.myAccount.phoneNumberNotVerified") }}
      </p>
      <BButton variant="primary" @click="phoneVerificationModal = true">
        <Icon name="ph:check" />
        {{ $t("dashboard.myAccount.verifyPhoneNumber") }}
      </BButton>
    </template>
    <p v-else>
      {{ $t("dashboard.myAccount.phoneNumberVerified") }}
    </p>
    <MyAccountPhoneVerificationModal 
      v-model="phoneVerificationModal" 
      @ok="$emit('refresh')" 
    />
  </BCard>
</template>