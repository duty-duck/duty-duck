<script setup lang="ts">
const model = defineModel<string>({ default: "Etc/UTC" });
const { t } = useI18n();
const localTimezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
const timezones = [
    { value: "Etc/UTC", text: t("dashboard.timezoneSelect.utcTimezoneLabel") },
    { value: localTimezone, text: t("dashboard.timezoneSelect.localTimezoneLabel", { timezone: localTimezone }) },
    ...Intl.supportedValuesOf('timeZone')
];
</script>

<template>
    <BFormSelect :options="timezones" v-model="model" size="sm" />
</template>