<script setup lang="ts">
import type { SignUpCommand } from "bindings/SignUpCommand";

const state = ref<"initial" | "success" | "error" | "conflict">("initial");
const localePath = useLocalePath();
const repo = useUserRepository();

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
      <BCardTitle>Thank you for registering!</BCardTitle>
      <p>
        You may now go to your personal dashboard to create your first monitors
      </p>
      <BButton :to="localePath('/dashboard')" variant="primary">Go to your dashboard</BButton>
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
      <BCardTitle>A user already exists with this e-mail address</BCardTitle>
      <p>
        It seems you already have a user account. You may use this account to
        access your dashboard.
      </p>
      <BButton to="/dashboard" variant="primary">Go to your dashboard</BButton>
    </BCard>
  </div>
  <div
    v-else
    class="container d-flex align-items-center"
    style="min-height: 100vh"
  >
    <div class="row gx-lg-6">
      <div class="col-lg-6 col-xl-5 pe-xl-5">
        <img src="/assets/navbar-duck.png" alt="Duty Duck Logo" height="80px" />
        <h1 class="mt-4 mb-3">Start monitoring your website today</h1>
        <p class="d-none d-md-block">
          Lorem ipsum dolor sit amet consectetur adipisicing elit. Rerum
          repudiandae laudantium cumque ex velit, autem fuga quaerat! Voluptas
          voluptate ipsa exercitationem reprehenderit, aspernatur perspiciatis
          non beatae quidem, maxime rem accusamus?
        </p>
        <p>
          <NuxtLink to="/" class="icon-link">
            <Icon name="ph:house-simple-duotone" size="20px" />
            Learn more
          </NuxtLink>
        </p>
      </div>
      <div class="col-lg-6">
        <BAlert variant="danger" :model-value="state == 'error'">
          Something went wrong and we were not able to create your account.
          Please try again.
        </BAlert>
        <SignUpForm @submit="onSubmit" />
      </div>
    </div>
  </div>
</template>
