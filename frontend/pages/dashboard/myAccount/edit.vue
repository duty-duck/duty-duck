<script setup lang="ts">
import useVuelidate from "@vuelidate/core";
import { required } from "@vuelidate/validators";
import PhoneInput from "~/components/PhoneInput.vue";

const auth = useAuthMandatory();
const localePath = useLocalePath();
const state = reactive({
  phoneNumber: { isValid: false, value: "f" },
});

const rules = {
  phoneNumber: { value: { required }, isValid: { isTrue: (b: boolean) => b } },
};

const v$ = useVuelidate(rules, state);
</script>
<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.sidebar.home")
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

      <div>
        {{ state }}
      </div>

      <BForm>
        <BFormGroup
          id="phoneNumberGroup"
          label="Mobile phone number"
          label-for="phoneNumberInput"
          description="Let us know your phone number"
          invalid-feedback="foo"
          :state="v$.phoneNumber.$dirty ? !v$.phoneNumber.$invalid : null"
        >
          <PhoneInput
            id="phoneNumberInput"
            :value="state.phoneNumber.value"
            @change="(data) => v$.phoneNumber.$model = data"
            @blur="v$.phoneNumber.$touch"
          />
        </BFormGroup>
      </BForm>
    </BContainer>
  </div>
</template>
