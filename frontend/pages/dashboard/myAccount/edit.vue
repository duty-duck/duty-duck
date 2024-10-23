<script setup lang="ts">
import useVuelidate from "@vuelidate/core";
import { email, helpers, required, sameAs } from "@vuelidate/validators";
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import PhoneInput from "~/components/PhoneInput.vue";

const { t } = useI18n();
const { show } = useToast();
const auth = await useAuthMandatory();
const userInfo = auth.userProfile.user;
const localePath = useLocalePath();
const repo = useUserRepository();

const state = reactive({
  firstName: userInfo.firstName || "",
  lastName: userInfo.lastName || "",
  email: userInfo.email || "",
  emailConfirmation: userInfo.email,
  password: "",
  passwordConfirmation: "",
  phoneNumber: {
    isValid: null as boolean | null,
    value: userInfo.phoneNumber || "",
    formattedNumber: userInfo.phoneNumber || "" as string | null
  },
});

const isStrongPassword = usePasswordValidator(
  computed(() => [state.firstName, state.lastName])
);

watch(
  () => state.email,
  (email) => {
    // if the e-mail is equal to the original e-mail (i.e. the user did not edit their e-mail),
    // then the e-mail confirmation must also be equal to original e-mail
    // also, notice that the e-mail confirmation input is only displayed when the e-mail is edited
    if (email == userInfo.email) {
      state.emailConfirmation = userInfo.email;
    }
  }
);

watch(
  () => state.password,
  (password) => {
    if (password == "") {
      state.passwordConfirmation = "";
    }
  }
);

const phoneNumberValidator = helpers.withMessage(
  "Invalid phone number",
  (b: boolean | null) => b == null || b
);
const rules = {
  firstName: { required },
  lastName: { required },
  email: { required, email },
  emailConfirmation: {
    required,
    sameAsEmail: sameAs(computed(() => state.email)),
  },
  password: { isStrongPassword },
  passwordConfirmation: {
    sameAsPassword: sameAs(computed(() => state.password)),
  },
  phoneNumber: {
    value: {},
    formattedNumber: {},
    isValid: { isValid: phoneNumberValidator },
  },
};

const v$ = useVuelidate(rules, state);

const onSubmit = async () => {
  if (!(await v$.value.$validate())) {
    show?.({
      props: {
        title: t("dashboard.myAccount.edit.invalidProfileToast.title"),
        body: t("dashboard.myAccount.edit.invalidProfileToast.body"),
      },
    });
    return;
  }

  const command: UpdateProfileCommand = {
    firstName: state.firstName != userInfo.firstName ? state.firstName : null,
    lastName: state.lastName != userInfo.lastName ? state.lastName : null,
    phoneNumber:
      state.phoneNumber.formattedNumber != userInfo.phoneNumber &&
      state.phoneNumber.formattedNumber != ""
        ? state.phoneNumber.formattedNumber
        : null,
    email: state.email != userInfo.email ? state.email : null,
    password: state.password != "" ? state.password : null,
  };

  const response = await repo.updateProfile(command);
  repo.refreshUserProfile();
  navigateTo(localePath("/dashboard/myAccount"));

  if (response.needsSessionInvalidation) {
    show?.({
      props: {
        title: t("dashboard.myAccount.edit.sessionInvalidationToast.title"),
        body: t("dashboard.myAccount.edit.sessionInvalidationToast.body"),
      },
    });
    
    setTimeout(() => {
      auth.logout();
    }, 8000)
  } else {
    show?.({
      props: {
        title: t("dashboard.myAccount.edit.successToast.title"),
        body: t("dashboard.myAccount.edit.successToast.body"),
      },
    });
  }
};
</script>
<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.mainSidebar.home")
        }}</BBreadcrumbItem>
        <BBreadcrumbItem :to="localePath('/dashboard/myAccount')">{{
          $t("dashboard.userMenu.myAccount")
        }}</BBreadcrumbItem>
        <BBreadcrumbItem active>{{
          $t("dashboard.myAccount.edit.pageTitle")
        }}</BBreadcrumbItem>
      </BBreadcrumb>

      <h2>
        <Icon name="ph:user" />
        {{ $t("dashboard.myAccount.edit.pageTitle") }}
      </h2>

      <BForm @submit.prevent="onSubmit">
        <BFormGroup
          class="mb-4"
          id="firstNameGroup"
          :label="$t('dashboard.myAccount.firstName')"
          label-for="firstNameInput"
          :invalid-feedback="v$.firstName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.firstName.$model"
            id="firstNameInput"
            :state="v$.firstName.$dirty ? !v$.firstName.$invalid : null"
          />
        </BFormGroup>
        <BFormGroup
          class="mb-4"
          id="lastNameGroup"
          :label="$t('dashboard.myAccount.lastName')"
          label-for="lastNameInput"
          :invalid-feedback="v$.lastName.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.lastName.$model"
            id="lastNameInput"
            :state="v$.lastName.$dirty ? !v$.lastName.$invalid : null"
          />
        </BFormGroup>
        <BFormGroup
          class="mb-4"
          id="emailGroup"
          :label="$t('dashboard.myAccount.email')"
          label-for="emailInput"
          :invalid-feedback="v$.email.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.email.$model"
            id="emailInput"
            :state="v$.email.$dirty ? !v$.email.$invalid : null"
          />
        </BFormGroup>
        <BFormGroup
          v-if="state.email != userInfo.email"
          class="mb-4"
          id="emailConfirmationGroup"
          :label="$t('dashboard.myAccount.emailConfirmation')"
          label-for="emailConfirmationInput"
          :invalid-feedback="
            v$.emailConfirmation.$errors[0]?.$message.toString()
          "
        >
          <BFormInput
            v-model="v$.emailConfirmation.$model"
            id="emailConfirmationInput"
            :state="
              v$.emailConfirmation.$dirty
                ? !v$.emailConfirmation.$invalid
                : null
            "
          />
        </BFormGroup>
        <BFormGroup
          class="mb-4"
          id="emailGroup"
          :label="$t('dashboard.myAccount.password')"
          label-for="passwordInput"
          :invalid-feedback="v$.password.$errors[0]?.$message.toString()"
        >
          <BFormInput
            v-model="v$.password.$model"
            id="passwordInput"
            type="password"
            :state="v$.password.$dirty ? !v$.password.$invalid : null"
          />
        </BFormGroup>
        <BFormGroup
          v-if="state.password != ''"
          class="mb-4"
          id="passwordConfirmationGroup"
          :label="$t('dashboard.myAccount.passwordConfirmation')"
          label-for="passwordConfirmationInput"
          :invalid-feedback="
            v$.passwordConfirmation.$errors[0]?.$message.toString()
          "
        >
          <BFormInput
            v-model="v$.passwordConfirmation.$model"
            type="password"
            id="passwordConfirmationInput"
            :state="
              v$.passwordConfirmation.$dirty
                ? !v$.passwordConfirmation.$invalid
                : null
            "
          />
        </BFormGroup>
        <BFormGroup
          class="mb-4"
          id="phoneNumberGroup"
          :label="$t('dashboard.myAccount.phoneNumber')"
          label-for="phoneNumberInput"
          :description="$t('dashboard.myAccount.phoneNumberDescription')"
          :invalid-feedback="v$.phoneNumber.$errors[0]?.$message.toString()"
          :state="v$.phoneNumber.$model.isValid"
        >
          <PhoneInput
            id="phoneNumberInput"
            :value="state.phoneNumber.value"
            @change="(data) => (v$.phoneNumber.$model = data)"
            @blur="v$.phoneNumber.$touch"
          />
        </BFormGroup>
        <BButton type="submit" class="icon-link" :disabled="v$.$invalid">
          <Icon name="ph:floppy-disk" />
          {{ $t("dashboard.myAccount.edit.save") }}
        </BButton>
      </BForm>
    </BContainer>
  </div>
</template>
