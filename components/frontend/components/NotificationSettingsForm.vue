<script lang="ts" setup>
export type NotificationSettings = {
    pushNotificationEnabled: boolean;
    smsNotificationEnabled: boolean;
    emailNotificationEnabled: boolean;
};

const { t } = useI18n();
const model = defineModel<NotificationSettings>({ required: true });
const options = [{
    text: t("dashboard.notificationSettingsForm.pushNotification"),
    value: "pushNotificationEnabled",
}, {
    text: t("dashboard.notificationSettingsForm.smsNotification"),
    value: "smsNotificationEnabled",
}, {
    text: t("dashboard.notificationSettingsForm.emailNotification"),
    value: "emailNotificationEnabled",
}];
const checkBoxModel = computed({
    get() {
        return [
            model.value.pushNotificationEnabled ? "pushNotificationEnabled" : null,
            model.value.smsNotificationEnabled ? "smsNotificationEnabled" : null,
            model.value.emailNotificationEnabled ? "emailNotificationEnabled" : null,
        ].filter(Boolean) as string[];
    },
    set(value: string[]) {
        model.value.pushNotificationEnabled = value.includes("pushNotificationEnabled");
        model.value.smsNotificationEnabled = value.includes("smsNotificationEnabled");
        model.value.emailNotificationEnabled = value.includes("emailNotificationEnabled");
    }
});
</script>

<template>
    <div>
        <BFormCheckboxGroup switches v-model="checkBoxModel" :options="options" />
    </div>
</template>