<!-- We are using a custom error boundary for the dashboard since the NuxtErrorBoundary provided by the framework is buggy and does not properly clear the error on route change.
See this issue https://github.com/nuxt/nuxt/issues/15781 for reference 
-->
<script setup lang="ts">
import type { NuxtError } from "#app";
const error = ref<NuxtError>();
const localePath = useLocalePath();

onErrorCaptured((err) => {
  console.error("Error boundary caught error:", err);
  error.value = err as NuxtError;
});

const clearError = () => {
  error.value = undefined;
};

const route = useRoute();
watch(
  () => route.fullPath,
  clearError
);
</script>

<template>
  <div v-if="!error">
    <slot />
  </div>
  <template v-else-if="error.statusCode == 404">
    <BCard class="text-center py-5">
      <Icon name="ph:magnifying-glass-duotone" size="10rem" class="m" />
      <h2>Not found</h2>
      <p>The resource you are looking for cannot be found.</p>
      <BButton @click="clearError" :to="localePath('/dashboard')">Go home</BButton>
    </BCard>
  </template>
  <template v-else-if="error.statusCode == 403">
    <BCard class="text-center py-5">
      <Icon name="ph:hand-palm-duotone" size="10rem" class="m" />
      <h2>Access denied</h2>
      <p>You are not authorized to access this resource.</p>
      <BButton @click="clearError" :to="localePath('/dashboard')">Go home</BButton>
    </BCard>
  </template>
  <template v-else>
    <BCard class="text-center py-5">
      <Icon name="ph:warning-duotone" size="10rem" class="m" />
      <h2>An error occurred</h2>
      <p>An error occurred: {{ error }}</p>
      <p>Please try again later or contact support if the problem persists.</p>
      <BButton @click="clearError" :to="localePath('/dashboard')">Go home</BButton>
    </BCard>
  </template>
</template>
