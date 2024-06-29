<script setup lang="ts">
    import { RouterLink } from 'vue-router';
</script>

<template>
    <div>
        <h1>Simple monitoring</h1>
        <p>
            <RouterLink to="/signup">Go to sign up</RouterLink>
            <RouterLink to="/dashboard">Go to dashboard</RouterLink>
        </p>
    </div>
</template>