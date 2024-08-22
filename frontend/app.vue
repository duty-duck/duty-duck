<script>
import "./assets/main.scss";
import 'intl-tel-input/build/css/intlTelInput.css';

if (import.meta.client) {
  import("bootstrap/dist/js/bootstrap");
}
</script>

<template>
  <NuxtLoadingIndicator color="#70BE4D" />
  <NuxtPage />
  <BToastOrchestrator />
</template>
