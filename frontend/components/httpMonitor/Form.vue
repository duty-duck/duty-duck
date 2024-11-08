<script lang="ts" setup>
import type { EntityMetadata } from 'bindings/EntityMetadata';
import type { NotificationSettings } from '../NotificationSettingsForm.vue';

export type HttpMonitorFormData = {
  url: string;
  intervalSeconds: number;
  metadata: EntityMetadata;
  recoveryConfirmationThreshold: number,
  downtimeConfirmationThreshold: number,
  notificationSettings: NotificationSettings,
}

const props = withDefaults(defineProps<HttpMonitorFormData>(), {
  url: "",
  intervalSeconds: 60,
  metadata: () => ({ records: {} }),
  recoveryConfirmationThreshold: 2,
  downtimeConfirmationThreshold: 1,
  notificationSettings: {
    pushNotificationEnabled: true,
    smsNotificationEnabled: false,
    emailNotificationEnabled: true,
  }
});
const emits = defineEmits<{
  (e: 'submit', form: HttpMonitorFormData): void;
}>();

const form = reactive({ ...props });
const formIsComplete = computed(
  () =>
    form.url &&
    (form.url.startsWith("http://") || form.url.startsWith("https://"))
);
</script>

<template>
  <BForm @submit.prevent="emits('submit', form)" @keypress.enter.prevent="" id="monitor-form">
    <!-- URL Group -->
    <div class="mb-5">
      <BFormGroup>
        <label class="h5" for="urlInput">URL</label>
        <BInput id="urlInput" type="text" v-model="form.url" placeholder="https://..." />
      </BFormGroup>
      <FormHelp :text="$t('dashboard.monitors.form.urlDescription')" />
    </div>

    <!-- Interval Group -->
    <div class="mb-5">
      <label class="h5">
        {{ $t("dashboard.monitors.form.refreshInterval") }}
      </label>
      <HttpMonitorIntervalInput :value="form.intervalSeconds" class="mb-3"
        @change="(interval) => (form.intervalSeconds = interval.seconds)" />
      <FormHelp :text="$t('dashboard.monitors.form.refreshIntervalDescription')" />
    </div>

    <!-- Confirmation Thresholds groups -->
    <div class="row mb-5">
      <div class="col-lg-6">
        <BFormGroup>
          <label class="h6">
            {{ $t("dashboard.monitors.form.downtimeConfirmationThreshold") }}
          </label>
          <BInput type="number" min="1" max="10" v-model.number="form.downtimeConfirmationThreshold" />
        </BFormGroup>
        <FormHelp :text="$t(
          'dashboard.monitors.form.downtimeConfirmationThresholdDescription'
        )
          " />
      </div>
      <div class="col-lg-6">
        <BFormGroup>
          <label class="h6">
            {{ $t("dashboard.monitors.form.recoveryConfirmationThreshold") }}
          </label>
          <BInput type="number" min="1" max="10" v-model.number="form.recoveryConfirmationThreshold" />
        </BFormGroup>
        <FormHelp :text="$t(
          'dashboard.monitors.form.recoveryConfirmationThresholdDescription'
        )
          " />
      </div>
    </div>

    <!-- Notification Settings -->
    <BFormGroup class="mb-5">
      <label class="h5">
        {{ $t("dashboard.monitors.form.notificationSettings") }}
      </label>
      <NotificationSettingsForm v-model="form.notificationSettings" />
      <FormHelp :text="$t('dashboard.monitors.form.notificationSettingsDescription')" />
    </BFormGroup>

    <!-- Metadata group -->
    <div class="mb-4">
      <BFormGroup>
        <label for="metadata-input" class="h5">
          {{ $t("dashboard.monitors.form.metadata") }}
        </label>
        <DashboardMetadataInput class="mb-3" id="metadata-input" v-model="form.metadata" />
      </BFormGroup>
      <FormHelp :text="$t('dashboard.monitors.form.metadataDescription')" />
    </div>

    <!-- Submit button -->
    <div>
      <BButton type="submit" class="icon-link" :disabled="!formIsComplete">
        <Icon name="ph:floppy-disk-back-duotone" aria-hidden />
        {{ $t("dashboard.monitors.form.save") }}
      </BButton>
    </div>
  </BForm>
</template>

<style scoped lang="scss">
#monitor-form {
  max-width: var(--bs-breakpoint-lg);
}
</style>