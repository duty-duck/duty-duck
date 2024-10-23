<script setup lang="ts">
import { BCard } from "bootstrap-vue-next";

const auth = await useAuth();
const localePath = useLocalePath();
const phoneVerificationModal = ref<boolean>(false);
</script>
<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.mainSidebar.home")
          }}</BBreadcrumbItem>
        <BBreadcrumbItem active>{{
          $t("dashboard.userMenu.myAccount")
          }}</BBreadcrumbItem>
      </BBreadcrumb>

      <h2 class="mb-3">
        <Icon name="ph:user" />
        {{ $t("dashboard.myAccount.pageTitle") }}
      </h2>
      <BCard :title="$t('dashboard.myAccount.myInfo')" class="mb-3">
        <dl>
          <dt>{{ $t("dashboard.myAccount.firstName") }}</dt>
          <dd>{{ auth.userProfile.user.firstName || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.lastName") }}</dt>
          <dd>{{ auth.userProfile.user.lastName || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.email") }}</dt>
          <dd>{{ auth.userProfile.user.email || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.phoneNumber") }}</dt>
          <dd>{{ auth.userProfile.user.phoneNumber || "--" }}</dd>
        </dl>
        <BButton variant="primary" :to="localePath('/dashboard/myAccount/edit')">
          <Icon name="ph:pencil" />
          {{ $t("dashboard.myAccount.editButton") }}
        </BButton>
      </BCard>
      <BCard class="mb-3">
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
      </BCard>
      <MyAccountPhoneVerificationModal v-model="phoneVerificationModal" />
      <MyAccountPushNotificationConsent />
    </BContainer>
  </div>
</template>
