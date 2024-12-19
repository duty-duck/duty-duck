<script setup lang="ts">
import { useIntervalFn } from '@vueuse/core';

const { tm } = useI18n();
const currentQuestionIndex = ref(0);
const questions = computed(() => tm('homepage.hero.headlines') as string[]);


useIntervalFn(() => {
    currentQuestionIndex.value = (currentQuestionIndex.value + 1) % questions.value.length;
}, 7000);
</script>

<template>
    <Transition name="fade" mode="out-in">
        <h1 class="question" :key="currentQuestionIndex">{{ questions[currentQuestionIndex] }}</h1>
    </Transition>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

h1.question {
    font-family: 'Poppins', sans-serif;
    font-weight: 900;
    font-size: 2.5rem;
    background: linear-gradient(to bottom, darken($primary, 5%), $primary);
    background-clip: text;
    color: transparent;

    @include media-breakpoint-up(md) {
        font-size: 3rem;
        line-height: 5rem;
    }
}
</style>