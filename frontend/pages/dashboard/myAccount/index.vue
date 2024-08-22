<script setup lang="ts">
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

      <h2>
        <Icon name="ph:user" />
        {{ $t("dashboard.myAccount.pageTitle") }}
      </h2>
      <h3 class="h4 mb-4">
        {{ $t("dashboard.myAccount.myInfo") }}
      </h3>
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
      <BButton variant="primary" :to="localePath('/dashboard/myAccount/edit')">
        <Icon name="ph:pencil" />
        {{ $t("dashboard.myAccount.editButton") }}
      </BButton>
    </BContainer>
  </div>
</template>
