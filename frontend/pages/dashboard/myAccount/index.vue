<script setup lang="ts">
import { BCard } from "bootstrap-vue-next";

const auth = useAuthMandatory();
const localePath = useLocalePath();
</script>
<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.sidebar.home")
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
          <dd>{{ auth.state.idToken.parsed["firstName"] || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.lastName") }}</dt>
          <dd>{{ auth.state.idToken.parsed["lastName"] || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.email") }}</dt>
          <dd>{{ auth.state.idToken.parsed["email"] || "--" }}</dd>
          <dt>{{ $t("dashboard.myAccount.phoneNumber") }}</dt>
          <dd>{{ auth.state.idToken.parsed["phoneNumber"] || "--" }}</dd>
        </dl>
        <BButton
          variant="primary"
          :to="localePath('/dashboard/myAccount/edit')"
        >
          <Icon name="ph:pencil" />
          {{ $t("dashboard.myAccount.editButton") }}
        </BButton>
      </BCard>
        <PushNotificationConsent />

    </BContainer>
  </div>
</template>
