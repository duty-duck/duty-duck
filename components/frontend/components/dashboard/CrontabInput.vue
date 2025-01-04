<script setup lang="ts">
import useVuelidate from '@vuelidate/core';
import { required } from '@vuelidate/validators';

import cronstrue from 'cronstrue';
import 'cronstrue/locales/fr';

const { locale } = useI18n();
const crontabString = defineModel<string>();
const formState = reactive({
    minute: crontabString.value?.split(' ')[0] ?? '*',
    hour: crontabString.value?.split(' ')[1] ?? '*',
    day: crontabString.value?.split(' ')[2] ?? '*',
    month: crontabString.value?.split(' ')[3] ?? '*',
    dayOfWeek: crontabString.value?.split(' ')[4] ?? '*',
});

const crontabRules = { 
    minute: { required, isValidCrontabComponent: isValidCrontabComponent(0, 59) },
    hour: { required, isValidCrontabComponent: isValidCrontabComponent(0, 23) },
    day: { required, isValidCrontabComponent: isValidCrontabComponent(1, 31) },
    month: { required, isValidCrontabComponent: isValidCrontabComponent(1, 12) },
    dayOfWeek: { required, isValidCrontabComponent: isValidCrontabComponent(0, 6) }
};

const v$ = useVuelidate(
    crontabRules,
    formState
);

watch(formState, (newVal) => {
    crontabString.value = `${newVal.minute} ${newVal.hour} ${newVal.day} ${newVal.month} ${newVal.dayOfWeek}`;
});

const humanReadableCron = computed(() => {
    return crontabString.value && !v$.value.$invalid ? cronstrue.toString(crontabString.value, { locale: locale.value, use24HourTimeFormat: true, verbose: true, throwExceptionOnParseError: false }) : '';
});
</script>

<template>
    <div>
        <div class="d-flex gap-2 mb-2">
            <BFormGroup :label="$t('dashboard.crontabInput.minute')" :invalid-feedback="v$.minute.$errors[0]?.$message.toString()">
                <BInput v-model="v$.minute.$model" :state="!v$.minute.$invalid" />
            </BFormGroup>
            <BFormGroup :label="$t('dashboard.crontabInput.hour')" :invalid-feedback="v$.hour.$errors[0]?.$message.toString()">
                <BInput v-model="v$.hour.$model" :state="!v$.hour.$invalid" />
            </BFormGroup>
            <BFormGroup :label="$t('dashboard.crontabInput.day')" :invalid-feedback="v$.day.$errors[0]?.$message.toString()">
                <BInput v-model="v$.day.$model" :state="!v$.day.$invalid" />
            </BFormGroup>
            <BFormGroup :label="$t('dashboard.crontabInput.month')" :invalid-feedback="v$.month.$errors[0]?.$message.toString()">
                <BInput v-model="v$.month.$model" :state="!v$.month.$invalid" />
            </BFormGroup>
            <BFormGroup :label="$t('dashboard.crontabInput.dayOfWeek')" :invalid-feedback="v$.dayOfWeek.$errors[0]?.$message.toString()">
                <BInput v-model="v$.dayOfWeek.$model" :state="!v$.dayOfWeek.$invalid" />
            </BFormGroup>
        </div>
        <div>
            {{ humanReadableCron }}
        </div>
    </div>
</template>
