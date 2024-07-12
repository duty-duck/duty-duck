<script>
import "./assets/main.scss";

if (import.meta.client) {
  import("bootstrap/dist/js/bootstrap");
}
</script>

<template>
  <NuxtLoadingIndicator color="#70BE4D" />
  <NuxtPage />
</template>

