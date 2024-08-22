<script setup lang="ts">
const { locale, locales } = useI18n()
const localePath = useLocalePath();
const auth = useAuthMandatory();
const username = computed(() => {
  if (auth.state?.status == "authenticated") {
    return (
      `${auth.state.idToken.parsed.firstName} ${auth.state.idToken.parsed.lastName}`
    );
  }
  return "";
});

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
        <span class="user-name">{{ username }}</span>
      </a>
      <ul class="dropdown-menu dropdown-menu-end">
        <li>
          <NuxtLink class="dropdown-item icon-link" :to="localePath('/dashboard/myAccount')">
            <Icon name="ph:user" aria-hidden />
            {{ $t('dashboard.userMenu.myAccount') }}
          </NuxtLink>
        </li>
        <li>
          <a class="dropdown-item icon-link" href="#">
            <Icon name="ph:users-four-duotone" aria-hidden />
            {{ $t('dashboard.userMenu.myOrg') }}
          </a>
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
          <a class="dropdown-item icon-link" @click="auth.logout()" style="cursor: pointer">
            <Icon name="ph:sign-out" />
            {{ $t('dashboard.userMenu.logOut') }}
          </a>
        </li>
      </ul>
    </li>
  </ul>
</template>
