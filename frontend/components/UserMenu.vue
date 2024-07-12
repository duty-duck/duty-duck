<script setup lang="ts">
import { useAuth } from "@/stores/auth";
const auth = useAuthMandatory();
const username = computed(() => {
  if (auth.state?.status == "authenticated") {
    return (
      auth.state.idToken.parsed["name"] ||
      auth.state.idToken.parsed["preferred_username"]
    );
  }
  return "";
});
</script>
<template>
  <ul class="navbar-nav ms-auto">
    <li class="nav-item dropdown" id="auth-menu">
      <a
        class="nav-link dropdown-toggle"
        href="#"
        role="button"
        data-bs-toggle="dropdown"
        aria-expanded="false"
      >
        <i data-feather="user"></i>
        <span class="user-name">{{ username }}</span>
      </a>
      <ul class="dropdown-menu dropdown-menu-end">
        <li>
          <a class="dropdown-item icon-link" href="#">
            <Icon name="ph:user" />
            My account
          </a>
        </li>
        <li>
          <a class="dropdown-item icon-link" href="#">
            <Icon name="ph:users-four-duotone" />
            My organization
          </a>
        </li>
        <li>
          <hr class="dropdown-divider" />
        </li>
        <li>
          <a
            class="dropdown-item icon-link"
            @click="auth.logout()"
            style="cursor: pointer"
          >
            <Icon name="ph:sign-out" />
            Log out
          </a>
        </li>
      </ul>
    </li>
  </ul>
</template>
