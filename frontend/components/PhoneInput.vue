<script setup lang="ts">
import fr from "intl-tel-input/i18n/fr";
import en from "intl-tel-input/i18n/en";
import intlTelInput, { Iti } from "intl-tel-input";
import { BFormInput, BFormInvalidFeedback } from "bootstrap-vue-next";

const { locale } = useI18n();
const { value } = defineProps<{
  value: string;
}>();
const emits = defineEmits<{
  change: [{ isValid: boolean; value: string; formattedNumber: string | null }];
  blur: [];
}>();

const input = ref(null);
const iti = ref<null | Iti>(null);

const onInput = (event: any) => {
  const isValid: boolean = iti.value!.isValidNumber()!;
  emits("change", {
    value: event.target.value,
    formattedNumber: isValid ? iti.value!.getNumber() : null,
    isValid,
  });
};

onMounted(() => {
  let localeFile = fr;
  if (locale.value == "en") {
    localeFile = en;
  }
  iti.value = intlTelInput(input.value!, {
    i18n: localeFile,
    utilsScript: "/intl-tel-input/utils.js",
  });
});
</script>
<template>
  <div>
    <input
      ref="input"
      class="form-control"
      @input="onInput"
      @blur="emits('blur')"
      :value="value"
    />
  </div>
</template>

<style>
.iti {
  display: block;
  --iti-path-flags-1x: url("/intl-tel-input/flags.webp");
  --iti-path-flags-2x: url("/intl-tel-input/flags@2x.webp");
  --iti-path-globe-1x: url("/intl-tel-input/globe.webp");
  --iti-path-globe-2x: url("/intl-tel-input/globe@2x.webp");
}
</style>
