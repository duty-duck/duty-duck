<script setup lang="ts">
const auth = await useAuth();
const localePath = useLocalePath();
</script>

<template>
  <BCard :title="$t('dashboard.myAccount.myInfo')">
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
</template>