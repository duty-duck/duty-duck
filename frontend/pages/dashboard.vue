<script setup lang="ts">
import { useBreakpoints, breakpointsBootstrapV5 } from "@vueuse/core";


const auth = useAuthMandatory();
const breakpoints = useBreakpoints(breakpointsBootstrapV5);
const lgOrLarger = breakpoints.greaterOrEqual("lg");
const messageHandler = useFirebaseMessageHandler();
const firebaseMessaging = useFirebaseMessaging();

onBeforeMount(() => {
  // Register message handler
  firebaseMessaging.onMessage(messageHandler);
});
</script>

<template>
  <template v-if="!auth?.isLoading && auth?.userProfile?.user">
    <div class="offcanvas offcanvas-start" tabindex="-1" id="dashboard-offcanvas">
      <div class="offcanvas-header">
        <button type="button" class="btn-close" data-bs-dismiss="offcanvas" aria-label="Close"></button>
      </div>
      <div class="offcanvas-body">
        <DashboardDynamicSidebar v-if="!lgOrLarger" />
      </div>
    </div>
    <div class="container-fluid g-0">
      <div class="row g-0">
        <div class="d-none d-lg-block d-flex sticky-top" id="dashboard-sidebar">
          <div id="dashboard-sidebar-content">
            <DashboardSidebarBrand />
            <DashboardDynamicSidebar v-if="lgOrLarger" />
          </div>
        </div>
        <div class="col">
          <nav class="navbar navbar-expand sticky-top" id="dashboard-navbar">
            <div class="container-fluid">
              <button class="navbar-toggler d-block d-lg-none" type="button" data-bs-toggle="offcanvas"
                data-bs-target="#dashboard-offcanvas" aria-controls="dashboard-offcanvas" aria-expanded="false"
                aria-label="Toggle navigation">
                <span class="navbar-toggler-icon"></span>
              </button>
              <UserMenu />
            </div>
          </nav>
          <div class="container-fluid py-2 px-2 mt-4 px-lg-4" id="dashboard-container">
            <DashboardErrorBoundary>
              <NuxtPage :transition="{ name: 'page', mode: 'out-in' }" />
            </DashboardErrorBoundary>
          </div>
        </div>
      </div>
    </div>
  </template>
</template>

<style lang="scss">
@import "~/assets/main.scss";

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

#dashboard-sidebar {
  height: 100vh;
  overflow-y: auto;
  overflow-x: hidden;
  width: 260px;
  // box-shadow: 3px 0px 4px rgba(0, 0, 0, .2);
  z-index: 100;
  background-color: transparent;
  transition: background-color 0.5s ease;

  &:hover {
    background-color: white;
  }

}
</style>
