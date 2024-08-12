<!-- We are using a custom error boundary for the dashboard since the NuxtErrorBoundary provided by the framework is buggy and does not properly clear the error on route change.
See this issue https://github.com/nuxt/nuxt/issues/15781 for reference 
-->
<script setup lang="ts">
import type { NuxtError } from '#app';
const error = ref<NuxtError>()
const localePath = useLocalePath();

onErrorCaptured(err => {
    error.value = err as NuxtError
    return false
})

const route = useRoute()
watch(
    () => route.fullPath,
    () => {
        error.value = undefined
    },
)
</script>

<template>
    <slot v-if="!error" />
    <template v-else-if="error.statusCode == 404">
        <BCard class="text-center py-5">
            <Icon name="ph:magnifying-glass-duotone" size="10rem" class="m" />
            <h2>Not found</h2>
            <p>The resource you are looking for cannot be found.</p>
            <BButton :to="localePath('/dashboard')">Go home</BButton>
        </BCard>
    </template>
    <template v-else>

        <p>An error occurred: {{ error }}</p>
    </template>

</template>