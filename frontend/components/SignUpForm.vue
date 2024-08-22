<script lang="ts" setup>
import { required, email, sameAs, helpers } from "@vuelidate/validators";
import { useVuelidate } from "@vuelidate/core";
import { type SignUpCommand } from "bindings/SignUpCommand";

const userRepo = useUserRepository();
const state = reactive({
  firstName: "",
  lastName: "",
  email: "",
  password: "",
  passwordConfirm: "",
  organizationName: "",
});
const emit = defineEmits<{
  submit: [command: SignUpCommand];
}>();

const passwordFieldState = computed(() => {
  if (!v$.value.password.$dirty || v$.value.password.$pending) {
    return null;
  }
  return !v$.value.password.$invalid;
});
const isStrongPassword = usePasswordValidator(
  computed(() => [state.firstName, state.lastName])
);

const rules = {
  firstName: { required },
  lastName: { required },
  email: { required, email },
  password: { required, isStrongPassword },
  passwordConfirm: {
    required,
    sameAsPassword: sameAs(computed(() => state.password)),
  },
  organizationName: { required },
};

const v$ = useVuelidate(rules, state);

const onSubmit = async () => {
  if (await v$.value.$validate()) {
    emit("submit", state);
  }
};
</script>
<template>
  <BForm @submit.prevent="onSubmit">
    <div class="row mb-4">
      <h2 class="h5">Tell us about yourself</h2>
      <div class="col-lg-6">
        <BFormGroup
          id="firstNameGroup"
          label="First Name"
          label-for="firstNameInput"
          description="Let us know your name"
          floating
          :invalid-feedback="v$.firstName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.firstName.$model"
            id="firstNameInput"
            placeholder="Enter your first name please"
            required
            :state="v$.firstName.$dirty ? !v$.firstName.$invalid : null"
          />
        </BFormGroup>
      </div>
      <div class="col-lg-6">
        <BFormGroup
          id="lastNameGroup"
          label="Last Name"
          label-for="lastNameInput"
          floating
          :invalid-feedback="v$.lastName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.lastName.$model"
            id="lastNameInput"
            placeholder="Enter your last name please"
            :state="v$.lastName.$dirty ? !v$.lastName.$invalid : null"
          />
        </BFormGroup>
      </div>
    </div>
    <BFormGroup
      id="emailGroup"
      class="mb-3"
      label="E-mail address"
      label-for="emailInput"
      description="We won't share your e-mail with anyone"
      floating
      :invalid-feedback="v$.email.$errors[0]?.$message.toString()"
    >
      <BFormInput
        v-model="v$.email.$model"
        id="emailInput"
        type="email"
        placeholder="Enter your email please"
        :state="v$.email.$dirty ? !v$.email.$invalid : null"
      />
    </BFormGroup>
    <div class="row mb-3">
      <div class="col-md-6">
        <BFormGroup
          id="passwordGroup"
          label="Password"
          label-for="passwordInput"
          floating
          description="We recommend using a strong, random password"
          :invalid-feedback="v$.password.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.password.$model"
            id="passwordInput"
            type="password"
            placeholder="Enter your password please"
            required
            :state="passwordFieldState"
          />
        </BFormGroup>
      </div>
      <div class="col-md-6">
        <BFormGroup
          id="passwordConfirmGroup"
          label="Password confirmation"
          label-for="passwordConfirmInput"
          floating
          :invalid-feedback="v$.passwordConfirm.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.passwordConfirm.$model"
            id="passwordConfirmInput"
            type="password"
            placeholder="Enter your password please"
            required
            :state="
              v$.passwordConfirm.$dirty ? !v$.passwordConfirm.$invalid : null
            "
          />
        </BFormGroup>
      </div>
    </div>
    <div class="mb-3">
      <h2 class="h5">Tell us about your oganization</h2>
      <p>
        After you sign up, you will be able to invite other members of your
        organization to collaborate. The name of your organization will show on
        e-mails and invoices.
      </p>
      <BFormGroup
        id="orgGroup"
        label="Your organization"
        label-for="orgInput"
        description="Don't worry, you will be able to rename your organization later"
        floating
      >
        <BFormInput
          v-model="v$.organizationName.$model"
          :state="
            v$.organizationName.$dirty ? !v$.organizationName.$invalid : null
          "
          id="orgInput"
          placeholder="Enter your organization's name"
          required
        />
      </BFormGroup>
    </div>
    <div>
      <BButton :disabled="v$.$invalid" type="submit">Sign up</BButton>
    </div>
  </BForm>
</template>
