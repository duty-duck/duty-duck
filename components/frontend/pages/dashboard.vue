<script setup lang="ts">
import { useBreakpoints, breakpointsBootstrapV5 } from "@vueuse/core";

await useAuth();
const breakpoints = useBreakpoints(breakpointsBootstrapV5);
const lgOrLarger = breakpoints.greaterOrEqual("lg");
const messageHandler = useFirebaseMessageHandler();
const firebaseMessaging = useFirebaseMessaging();
const showOffcanvas = ref(false);
const route = useRoute();

// Close offcanvas when route changes
watch(route, () => {
  showOffcanvas.value = false;
});

onBeforeMount(() => {
  // Register message handler
  firebaseMessaging.onMessage(messageHandler);
});
</script>

<template>
  <BOffcanvas v-model="showOffcanvas" placement="start" v-if="!lgOrLarger" body-class="offcanvas-body-container">
    <DashboardSidebarBrand textAlwaysVisible />
    <DashboardDynamicSidebar style="flex-grow: 1" v-if="!lgOrLarger" />
  </BOffcanvas>
  <div class="container-fluid g-0">
    <div class="row g-0">
      <div class="d-none d-lg-block d-flex sticky-top" id="dashboard-sidebar">
        <div id="dashboard-sidebar-content">
          <DashboardSidebarBrand />
          <DashboardDynamicSidebar style="flex-grow: 1" v-if="lgOrLarger" />
        </div>
      </div>
      <div class="col">
        <nav class="navbar navbar-expand sticky-top" id="dashboard-navbar">
          <div class="container-fluid">
            <button class="navbar-toggler d-block d-lg-none" type="button" @click="showOffcanvas = true"
              aria-label="Toggle navigation">
              <span class="navbar-toggler-icon"></span>
            </button>
            <UserMenu />
          </div>
        </nav>
        <div id="dashboard-container">
          <DashboardErrorBoundary>
            <NuxtPage :fall :transition="{ name: 'page', mode: 'out-in' }" />
          </DashboardErrorBoundary>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
@import "~/assets/main.scss";

.offcanvas-body-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: absolute;
  top: 0;
  left: 0;
  right: 30px;
}

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
  width: 100px;
  z-index: 100;
  background-color: white;
  border-right: 1px solid rgb(234 236 241);

  @include media-breakpoint-up(xxl) {
    width: 260px;
  }

  #dashboard-sidebar-content {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .nav-item {
    width: 100%;
  }

  .nav-link {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    font-size: 0.76rem;
  }

  @include media-breakpoint-up(xxl) {
    .nav-link {
      flex-direction: row;
      justify-content: flex-start;
    }

    .nav-link {
      font-size: 1rem;
    }
  }
}

#dashboard-container {
  @extend .container-fluid;
  @extend .pt-2;
  @extend .px-1;
  @extend .mt-4;
  @extend .px-xxl-4;
  padding-bottom: 5rem;
}

.navbar .navbar-collapse.show {
  padding-bottom: 1rem;

  @include media-breakpoint-up(lg) {
    padding-bottom: 0;
  }
}

#dashboard-navbar {
  height: $navbar-height;
  @include blurry-gray-background;
  z-index: 1;

  #auth-menu {
    height: $navbar-height;
    display: flex;
    align-items: center;
  }

  .user-name {
    font-size: $font-size-base * 0.8;
  }
}
</style>
