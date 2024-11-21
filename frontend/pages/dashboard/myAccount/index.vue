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
      <div class="d-flex flex-column gap-3">
      <MyAccountUserInfo />
      <MyAccountPhoneVerification @refresh="auth.refreshUserProfile()" />
      <MyAccountPhoneVerificationModal v-model="phoneVerificationModal" @ok="auth.refreshUserProfile()" />
      <MyAccountPushNotificationConsent />
      <MyAccountApiTokens />
      </div>
    </BContainer>
  </div>
</template>
