<script setup lang="ts">
    const localePath = useLocalePath();
</script>
<template>
    <div>
        <h1>Simple monitoring</h1>
        <p>
            <NuxtLink :to="localePath('/signup')">Go to sign up</NuxtLink>
            <NuxtLink :to="localePath('/dashboard')">Go to dashboard</NuxtLink>
        </p>
    </div>
</template>