<script lang="ts" setup>
import { required, email, sameAs, helpers } from "@vuelidate/validators";
import { useVuelidate } from "@vuelidate/core";
import { type SignUpCommand } from "bindings/SignUpCommand";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

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
      <h2 class="h5">{{ $t('signup.form.aboutYourself') }}</h2>
      <div class="col-lg-6">
        <BFormGroup
          id="firstNameGroup"
          :label="$t('signup.form.firstName')"
          label-for="firstNameInput"
          :description="$t('signup.form.nameDescription')"
          floating
          :invalid-feedback="v$.firstName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.firstName.$model"
            id="firstNameInput"
            :placeholder="$t('signup.form.firstNamePlaceholder')"
            required
            :state="v$.firstName.$dirty ? !v$.firstName.$invalid : null"
          />
        </BFormGroup>
      </div>
      <div class="col-lg-6">
        <BFormGroup
          id="lastNameGroup"
          :label="$t('signup.form.lastName')"
          label-for="lastNameInput"
          floating
          :invalid-feedback="v$.lastName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.lastName.$model"
            id="lastNameInput"
            :placeholder="$t('signup.form.lastNamePlaceholder')"
            :state="v$.lastName.$dirty ? !v$.lastName.$invalid : null"
          />
        </BFormGroup>
      </div>
    </div>
    <BFormGroup
      id="emailGroup"
      class="mb-3"
      :label="$t('signup.form.email')"
      label-for="emailInput"
      :description="$t('signup.form.emailDescription')"
      floating
      :invalid-feedback="v$.email.$errors[0]?.$message.toString()"
    >
      <BFormInput
        v-model="v$.email.$model"
        id="emailInput"
        type="email"
        :placeholder="$t('signup.form.emailPlaceholder')"
        :state="v$.email.$dirty ? !v$.email.$invalid : null"
      />
    </BFormGroup>
    <div class="row mb-3">
      <div class="col-md-6">
        <BFormGroup
          id="passwordGroup"
          :label="$t('signup.form.password')"
          label-for="passwordInput"
          floating
          :description="$t('signup.form.passwordDescription')"
          :invalid-feedback="v$.password.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.password.$model"
            id="passwordInput"
            type="password"
            :placeholder="$t('signup.form.passwordPlaceholder')"
            required
            :state="passwordFieldState"
          />
        </BFormGroup>
      </div>
      <div class="col-md-6">
        <BFormGroup
          id="passwordConfirmGroup"
          :label="$t('signup.form.passwordConfirm')"
          label-for="passwordConfirmInput"
          floating
          :invalid-feedback="v$.passwordConfirm.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.passwordConfirm.$model"
            id="passwordConfirmInput"
            type="password"
            :placeholder="$t('signup.form.passwordConfirmPlaceholder')"
            required
            :state="
              v$.passwordConfirm.$dirty ? !v$.passwordConfirm.$invalid : null
            "
          />
        </BFormGroup>
      </div>
    </div>
    <div class="mb-3">
      <h2 class="h5">{{ $t('signup.form.aboutOrganization') }}</h2>
      <p>{{ $t('signup.form.organizationDescription') }}</p>
      <BFormGroup
        id="orgGroup"
        :label="$t('signup.form.organization')"
        label-for="orgInput"
        :description="$t('signup.form.organizationRename')"
        floating
      >
        <BFormInput
          v-model="v$.organizationName.$model"
          :state="
            v$.organizationName.$dirty ? !v$.organizationName.$invalid : null
          "
          id="orgInput"
          :placeholder="$t('signup.form.organizationPlaceholder')"
          required
        />
      </BFormGroup>
    </div>
    <div>
      <BButton :disabled="v$.$invalid" type="submit">{{ $t('signup.form.submit') }}</BButton>
    </div>
  </BForm>
</template>
