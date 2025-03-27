<script setup lang="ts">
const { locale, locales } = useI18n()
const localePath = useLocalePath();
const { userName, logout } = await useAuth();
const { userHasPermissionComputed } = await useAuth();

const canListOrganizationMembers = userHasPermissionComputed('listOrganizationMembers');
const switchLocalePath = useSwitchLocalePath()
const availableLocales = computed(() => {
  return (locales.value).filter(i => i.code !== locale.value)
})
</script>

<template>
  <ul class="navbar-nav ms-auto">
    <BNavItemDropdown toggle-class="d-flex gap-2 align-items-center">
      <template #button-content>
        <UserAvatar />
        <span class="user-name">{{ userName }}</span>
      </template>
      <template #default>
        <BDropdownItem>
          <NuxtLink class="icon-link icon-link text-reset text-decoration-none"
            :to="localePath('/dashboard/myAccount')">
            <Icon name="ph:user" aria-hidden />
            {{ $t('dashboard.userMenu.myAccount') }}
          </NuxtLink>
        </BDropdownItem>
        <BDropdownItem>
          <NuxtLink class="icon-link icon-link text-reset text-decoration-none" :to="localePath('/dashboard/myOrg')"
            v-if="canListOrganizationMembers">
            <Icon name="ph:users-four-duotone" aria-hidden />
            {{ $t('dashboard.userMenu.myOrg') }}
          </NuxtLink>
        </BDropdownItem>
        <BDropdownDivider />
        <BDropdownItem v-for="locale in availableLocales" :key="locale.code">
          <NuxtLink :to="switchLocalePath(locale.code)" class="icon-link text-reset text-decoration-none">
            <Icon name="ph:translate" aria-label="Language selection" />
            {{ locale.name }}
          </NuxtLink>
        </BDropdownItem>
        <BDropdownDivider />
        <BDropdownItem class="icon-link" @click="logout()" style="cursor: pointer">
          <Icon name="ph:sign-out" />
          {{ $t('dashboard.userMenu.logOut') }}
        </BDropdownItem>
      </template>
    </BNavItemDropdown>
  </ul>
</template>