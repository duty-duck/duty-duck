<script lang="ts" setup>
import type { EntityMetadata } from 'bindings/EntityMetadata';
import type { NotificationSettings } from '../NotificationSettingsForm.vue';
import type { RequestHeaders } from 'bindings/RequestHeaders';

export type HttpMonitorFormData = {
  url: string;
  intervalSeconds: number;
  metadata: EntityMetadata;
  recoveryConfirmationThreshold: number,
  downtimeConfirmationThreshold: number,
  notificationSettings: NotificationSettings,
  requestHeaders: RequestHeaders,
  requestTimeoutMs: number,
}

// Define the props for the component
// The props are used to populate the form when first rendered
const props = withDefaults(defineProps<HttpMonitorFormData>(), {
  url: "",
  intervalSeconds: 60,
  metadata: () => ({ records: {} }),
  requestHeaders: () => ({ headers: {} }),
  requestTimeoutMs: 10000,
  recoveryConfirmationThreshold: 2,
  downtimeConfirmationThreshold: 1,
  notificationSettings: {
    pushNotificationEnabled: true,
    smsNotificationEnabled: false,
    emailNotificationEnabled: true,
  }
});

// Define the external events the form will emit
const emits = defineEmits<{
  (e: 'submit', form: HttpMonitorFormData): void;
}>();

// Here's the internal state of the form, populated from the props
const form = reactive({ ...props });

// A computed property that determines if the form is valid
// We could use a validation library such as Vuelidate here but this will do for now
const formIsComplete = computed(
  () =>
    form.url &&
    (form.url.startsWith("http://") || form.url.startsWith("https://"))
);

const requestTimeoutSeconds = computed({
  get: () => props.requestTimeoutMs / 1000,
  set: (value: number) => form.requestTimeoutMs = value * 1000,
});
</script>

<template>
  <BForm @submit.prevent="emits('submit', form)" @keypress.enter.prevent="" id="monitor-form">
    <!-- URL Group -->
    <div class="mb-4">
      <BFormGroup>
        <label class="h5" for="urlInput">URL</label>
        <BInput id="urlInput" type="text" v-model="form.url" placeholder="https://..." />
      </BFormGroup>
      <FormHelp :text="$t('dashboard.monitors.form.urlDescription')" />
    </div>

    <!-- Interval Group -->
    <div class="mb-4">
      <label class="h5">
        {{ $t("dashboard.monitors.form.refreshInterval") }}
      </label>
      <HttpMonitorIntervalInput :value="form.intervalSeconds" class="mb-3"
        @change="(interval) => (form.intervalSeconds = interval.seconds)" />
      <FormHelp :text="$t('dashboard.monitors.form.refreshIntervalDescription')" />
    </div>

    <h5 class="mb-3">
      {{ $t("dashboard.monitors.form.advancedSettings") }}
    </h5>

    <BAccordion flush class="mb-3">
      <BAccordionItem :title="$t('dashboard.monitors.form.thresholds')">
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
      </BAccordionItem>
      <BAccordionItem :title="$t('dashboard.monitors.form.notificationSettings')">
        <!-- Notification Settings -->
        <BFormGroup class="mb-5">
          <NotificationSettingsForm v-model="form.notificationSettings" />
          <FormHelp :text="$t('dashboard.monitors.form.notificationSettingsDescription')" />
        </BFormGroup>
      </BAccordionItem>

      <BAccordionItem :title="$t('dashboard.monitors.form.requestSettings')">
        <!-- Request Timeout group -->
        <div class="mb-4">
          <BFormGroup>
            <label for="request-timeout-input" class="h6">
              {{ $t("dashboard.monitors.form.requestTimeout") }}
            </label>
            <div class="d-flex align-items-center gap-3">
              <BInput type="number" min="1" max="15" v-model.number="requestTimeoutSeconds" style="max-width: 100px;" />
              <span class="ms-2"> {{ $t("dashboard.monitors.form.seconds") }}</span>
            </div>
          </BFormGroup>
          <FormHelp :text="$t('dashboard.monitors.form.requestTimeoutDescription')" />
        </div>

        <!-- Request Headers group -->
        <div class="mb-4">
          <BFormGroup>
            <label for="headers-input" class="h6">
              {{ $t("dashboard.monitors.form.requestHeaders") }}
            </label>
            <HttpMonitorRequestHeadersInput class="mb-3" id="headers-input" v-model="form.requestHeaders" />
          </BFormGroup>
          <FormHelp :text="$t('dashboard.monitors.form.requestHeadersDescription')" />
        </div>
      </BAccordionItem>

      <BAccordionItem :title="$t('dashboard.monitors.form.metadata')">
        <BFormGroup>
          <DashboardMetadataInput class="mb-3" id="metadata-input" v-model="form.metadata" />
        </BFormGroup>
        <FormHelp :text="$t('dashboard.monitors.form.metadataDescription')" />
      </BAccordionItem>
    </BAccordion>

    <!-- Submit button -->
    <div>
      <BButton type="submit" class="icon-link" :disabled="!formIsComplete">
        <Icon name="ph:floppy-disk-back-duotone" aria-hidden />
        {{ $t("dashboard.monitors.form.save") }}
      </BButton>
    </div>
  </BForm>
</template>

<style lang="scss">
#monitor-form {
  max-width: var(--bs-breakpoint-lg);

  .accordion-item,
  .accordion-button {
    background-color: transparent;
  }

}
</style>
