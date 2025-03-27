<script lang="ts" setup>
import "./assets/main.scss";
import 'intl-tel-input/build/css/intlTelInput.css';
</script>

<template>
  <BToastOrchestrator />
  <NuxtPwaManifest />
  <NuxtLoadingIndicator color="#05c46b" />
  <NuxtPage />
</template>
