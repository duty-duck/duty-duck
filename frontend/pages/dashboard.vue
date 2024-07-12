<script setup lang="ts">
import { RouterView } from "vue-router";
import { useAuthMandatory } from "@/stores/auth";
import { useBreakpoints, breakpointsBootstrapV5 } from "@vueuse/core";
import SideBar from "@/components/SideBar.vue";
import UserMenu from "@/components/UserMenu.vue";

const auth = useAuthMandatory();

const breakpoints = useBreakpoints(breakpointsBootstrapV5);
const lgOrLarger = breakpoints.greaterOrEqual("lg");
</script>

<template>
  <template v-if="auth.isReady && auth.isAuthenticated">
    <div
      class="offcanvas offcanvas-start"
      tabindex="-1"
      id="dashboard-offcanvas"
    >
      <div class="offcanvas-header">
        <button
          type="button"
          class="btn-close"
          data-bs-dismiss="offcanvas"
          aria-label="Close"
        ></button>
      </div>
      <div class="offcanvas-body">
        <SideBar v-if="!lgOrLarger"></SideBar>
      </div>
    </div>
    <div class="container-fluid g-0">
      <div class="row g-0">
        <div
          class="d-none d-lg-block col-lg-3 col-xl-2 d-flex"
          id="dashboard-sidebar"
        >
          <div id="dashboard-sidebar-content">
            <div id="sidebar-brand">
              <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" />
              <span>Duty Duck</span>
            </div>
            <SideBar v-if="lgOrLarger"></SideBar>
          </div>
        </div>
        <div class="col-lg-9 col-xl-10">
          <nav class="navbar navbar-expand sticky-top" id="dashboard-navbar">
            <div class="container-fluid">
              <button
                class="navbar-toggler d-block d-lg-none"
                type="button"
                data-bs-toggle="offcanvas"
                data-bs-target="#dashboard-offcanvas"
                aria-controls="dashboard-offcanvas"
                aria-expanded="false"
                aria-label="Toggle navigation"
              >
                <span class="navbar-toggler-icon"></span>
              </button>
              <UserMenu />
            </div>
          </nav>
          <div
            class="container-fluid py-2 px-2 mt-4 px-lg-4"
            id="dashboard-container"
          >
            <NuxtErrorBoundary>
              <NuxtPage :transition="{ name: 'page', mode: 'out-in' }" />
              <template #error="{ error }">
                <p>An error occurred: {{ error }}</p>
              </template>
            </NuxtErrorBoundary>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: all 0.1s ease-in-out;
}

.page-enter-from {
  opacity: 0;
  transform: translateX(10%);
}

.page-leave-to {
  opacity: 0;
}
</style>
