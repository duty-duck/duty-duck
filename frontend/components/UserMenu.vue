<script setup lang="ts">
const { locale, locales } = useI18n()
const localePath = useLocalePath();
const {userName, logout} = useAuthMandatory();
const {canComputed} = useAuth();

const canListOrganizationMembers = canComputed('listOrganizationMembers');
const switchLocalePath = useSwitchLocalePath()
const availableLocales = computed(() => {
  return (locales.value).filter(i => i.code !== locale.value)
})
</script>
<template>
  <ul class="navbar-nav ms-auto">
    <li class="nav-item dropdown" id="auth-menu">
      <a class="nav-link dropdown-toggle" href="#" role="button" data-bs-toggle="dropdown" aria-expanded="false">
        <i data-feather="user"></i>
        <span class="user-name">{{ userName }}</span>
      </a>
      <ul class="dropdown-menu dropdown-menu-end">
        <li>
          <NuxtLink class="dropdown-item icon-link" :to="localePath('/dashboard/myAccount')">
            <Icon name="ph:user" aria-hidden />
            {{ $t('dashboard.userMenu.myAccount') }}
          </NuxtLink>
        </li>
        <li>
          <NuxtLink class="dropdown-item icon-link" :to="localePath('/dashboard/myOrg')" v-if="canListOrganizationMembers">
            <Icon name="ph:users-four-duotone" aria-hidden />
            {{ $t('dashboard.userMenu.myOrg') }}
          </NuxtLink>
        </li>
        <li>
          <hr class="dropdown-divider" />
        </li>
        <li>
          <NuxtLink v-for="locale in availableLocales" :key="locale.code" :to="switchLocalePath(locale.code)" class="dropdown-item icon-link">
            <Icon name="ph:translate" aria-label="Language selection" />
            {{ locale.name }}
          </NuxtLink>

        </li>
        <li>
          <hr class="dropdown-divider" />
        </li>
        <li>
          <a class="dropdown-item icon-link" @click="logout()" style="cursor: pointer">
            <Icon name="ph:sign-out" />
            {{ $t('dashboard.userMenu.logOut') }}
          </a>
        </li>
      </ul>
    </li>
  </ul>
</template>
