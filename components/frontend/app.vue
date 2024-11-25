<script lang="ts" setup>
import "./assets/main.scss";
import 'intl-tel-input/build/css/intlTelInput.css';

if (import.meta.client) {
  // @ts-ignore
  import("bootstrap/dist/js/bootstrap");
}

const config = useRuntimeConfig();
</script>

<template>
  <NuxtPwaManifest />
  <NuxtLoadingIndicator color="#05c46b" />
  <NuxtPage />
  <BToastOrchestrator />
</template>
