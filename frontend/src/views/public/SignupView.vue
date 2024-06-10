<script setup lang="ts">
import { useAuth } from '@/stores/auth';
import { useRouter } from 'vue-router';

const auth = useAuth();
const router = useRouter();

auth.onReady(() => {
    if (auth.isAuthenticated) {
        router.replace('dashboard');
    }
})
</script>
<template>
    foo
</template>