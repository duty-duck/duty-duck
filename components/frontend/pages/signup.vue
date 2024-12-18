<script setup lang="ts">
import type { SignUpCommand } from "bindings/SignUpCommand";

const state = ref<"initial" | "success" | "error" | "conflict">("initial");
const localePath = useLocalePath();
const repo = usePublicUserRepository();

const onSubmit = async (command: SignUpCommand) => {
  state.value = await repo.signUp(command);
};
</script>
<template>
  <div
    v-if="state == 'success'"
    class="d-flex align-items-center justify-content-center"
    style="min-height: 100vh"
  >
    <BCard>
      <div class="text-center text-primary mb-4">
        <Icon name="ph:check-circle-duotone" size="128" />
      </div>
      <BCardTitle>{{ $t('signup.success.title') }}</BCardTitle>
      <p>{{ $t('signup.success.message') }}</p>
      <BButton :to="localePath('/dashboard')" variant="primary">{{ $t('signup.success.button') }}</BButton>
    </BCard>
  </div>
  <div
    v-else-if="state == 'conflict'"
    class="d-flex align-items-center justify-content-center"
    style="min-height: 100vh"
  >
    <BCard>
      <div class="text-center text-primary mb-4">
        <Icon name="ph:check-circle-duotone" size="128" />
      </div>
      <BCardTitle>{{ $t('signup.conflict.title') }}</BCardTitle>
      <p>{{ $t('signup.conflict.message') }}</p>
      <BButton :to="localePath('/dashboard')" variant="primary">{{ $t('signup.conflict.button') }}</BButton>
    </BCard>
  </div>
  <div
    v-else
    class="container d-flex align-items-center"
    style="min-height: 100vh"
  >
    <div class="row gx-lg-6">
      <div class="col-lg-6 col-xl-5 pe-xl-5">
        <img src="/assets/navbar-duck.png" :alt="$t('signup.altText.logo')" height="80px" />
        <h1 class="mt-4 mb-3">{{ $t('signup.intro.title') }}</h1>
        <p class="d-none d-md-block">{{ $t('signup.intro.description') }}</p>
        <p>
          <NuxtLink :to="localePath('/')" class="icon-link">
            <Icon name="ph:house-simple-duotone" size="20px" />
            {{ $t('signup.intro.learnMore') }}
          </NuxtLink>
        </p>
      </div>
      <div class="col-lg-6">
        <BAlert variant="danger" :model-value="state == 'error'">
          {{ $t('signup.error.message') }}
        </BAlert>
        <SignUpForm @submit="onSubmit" />
      </div>
    </div>
  </div>
</template>
