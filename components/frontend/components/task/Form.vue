<script setup lang="ts">
import { ProseP } from '#build/components';
import useVuelidate from '@vuelidate/core';
import { integer, minValue, required, requiredIf } from '@vuelidate/validators';
import type { EntityMetadata } from 'bindings/EntityMetadata';
import type { NotificationSettings } from '../NotificationSettingsForm.vue';

/**
 * A type that represents the data for a task form, used as both
 * the props and the state of the component. The props are used to populate the form when first rendered.
 */
export type TaskFormData = {
  id: string;
  name: string;
  description: string | null
  cronSchedule: string | null;
  sheduleTimezone: string | null;
  startWindowSeconds: number | null;
  latenessWindowSeconds: number | null;
  heartbeatTimeoutSeconds: number;
  notificationSettings: NotificationSettings;
  metadata: EntityMetadata,
}

type TaskFormProps = {
  data: TaskFormData
}

// Define the props for the component
// The props are used to populate the form when first rendered
const { data = {
  id: "",
  name: "",
  description: "",
  sheduleTimezone: null,
  cronSchedule: null,
  startWindowSeconds: 30,
  latenessWindowSeconds: 120,
  heartbeatTimeoutSeconds: 20,
  notificationSettings: {
    pushNotificationEnabled: true,
    smsNotificationEnabled: false,
    emailNotificationEnabled: true,
  },
  metadata: { records: {} }
}
} = defineProps<TaskFormProps>();


// Define the external events the form will emit
const emits = defineEmits<{
  (e: 'submit', form: TaskFormData): void;
}>();

// Here's the internal state of the form, populated from the props
const form = reactive({ ...data });

// Check if the task is new when the form is mounted
const isNewTask = !data.id;

// Define the form rules for validation
const taskIdAvailable = useTaskIdAvailableValidator();
const rules = {
  id: {
    required,
    mustBeValidId: (id: string) => /^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]$/.test(id),
    // If the task is new, we need to check if the task id is available
    // If the form is used to update an existing task, we don't need to check if the task id is available
    mustBeAvailable: isNewTask ? taskIdAvailable : () => true
  },
  name: { required },
  description: {},
  startWindowSeconds: { requiredIfScheduled: requiredIf(() => form.cronSchedule !== null), integer, minValue: minValue(10) },
  latenessWindowSeconds: { requiredIfScheduled: requiredIf(() => form.cronSchedule !== null), integer, minValue: minValue(10) },
  heartbeatTimeoutSeconds: { required, integer, minValue: minValue(5) },
  cronSchedule: { isValidCrontab: (value: string | null) => value ? isValidCrontab(value) : true }
};

const onScheduleToggle = (value: any) => {
  if (!value) {
    form.cronSchedule = null;
    form.startWindowSeconds = null;
    form.latenessWindowSeconds = null;
  } else if (form.cronSchedule === null) {
    form.cronSchedule = '* * * * *';
    form.startWindowSeconds = 30;
    form.latenessWindowSeconds = 120;
  }
}

const onSubmit = () => {
  if (!v$.value.$invalid) {
    emits('submit', form);
  }
}

const v$ = useVuelidate(rules, form);

// Automatically update the ID when the name changes
watch(() => form.name, (name) => {
  v$.value.id.$model = name.trim().toLowerCase().replace(/[^a-z0-9]/g, '-');
});
</script>

<template>
  <BForm id="task-form" @submit.prevent="onSubmit">
    <!-- Name Group -->
    <div class="mb-4">
      <BFormGroup :invalid-feedback="v$.name.$errors[0]?.$message.toString()">
        <label class="h5" for="nameInput">{{ $t('dashboard.tasks.form.name') }}</label>
        <BInput id="nameInput" type="text" v-model="v$.name.$model"
          :state="v$.name.$dirty ? !v$.name.$invalid : null" />
      </BFormGroup>
      <FormHelp :text="$t('dashboard.tasks.form.nameDescription')" />
    </div>


    <!-- Description Group -->
    <div class="mb-4">
      <BFormGroup>
        <label class="h5" for="descriptionInput">{{ $t('dashboard.tasks.form.description') }}</label>
        <BFormTextarea id="descriptionInput" type="text" v-model="v$.description.$model"
          :state="v$.description.$dirty ? !v$.description.$invalid : null" />
      </BFormGroup>
    </div>

    <!-- Schedule Section -->
    <section class="mb-5">
      <!-- Schedule toggle -->
      <div class="mb-4">
        <label class="h5">{{ $t('dashboard.tasks.form.schedule') }}</label>
        <BFormCheckbox :model-value="!!v$.cronSchedule.$model" switch @update:model-value="onScheduleToggle">
          {{ $t('dashboard.tasks.form.isScheduledTask') }}
        </BFormCheckbox>

      </div>

      <div v-if="v$.cronSchedule.$model">
        <!-- Schedule group -->
        <BFormGroup class="mb-4">
          <label class="h6">{{ $t('dashboard.tasks.form.scheduleInputLabel') }}</label>
          <DashboardCrontabInput v-model="v$.cronSchedule.$model" />
          <FormHelp :text="$t('dashboard.tasks.form.scheduleDescription')" />
        </BFormGroup>

        <div class="row">
          <div class="col-md-6">
            <BFormGroup>
              <label class="h6">{{ $t('dashboard.tasks.form.startWindow') }}</label>
              <div class="d-flex align-items-center gap-1">
                <BInput min="10" style="width: 100px;" type="number" v-model.number="v$.startWindowSeconds.$model" />
                <span class="ms-2">{{ $t('dashboard.tasks.form.seconds') }}</span>
              </div>
              <FormHelp :text="$t('dashboard.tasks.form.startWindowDescription')" />
            </BFormGroup>
          </div>
          <div class="col-md-6">
            <BFormGroup>
              <label class="h6">{{ $t('dashboard.tasks.form.latenessWindow') }}</label>
              <div class="d-flex align-items-center gap-1">
                <BInput min="10" style="width: 100px;" type="number" v-model.number="v$.latenessWindowSeconds.$model" />
                <span class="ms-2">{{ $t('dashboard.tasks.form.seconds') }}</span>
              </div>
              <FormHelp :text="$t('dashboard.tasks.form.latenessWindowDescription')" />
            </BFormGroup>
          </div>
        </div>
      </div>
      <!-- Help displayed only if not a scheduled task -->
      <FormHelp v-else :text="$t('dashboard.tasks.form.isScheduledTaskDescription')" />
    </section>


    <!-- Advanced settings accordion -->
    <h5 class="mb-3">{{ $t('dashboard.tasks.form.advancedSettings') }}</h5>
    <BAccordion flush class="mb-3">
      <!-- ID Group -->
      <BAccordionItem :title="$t('dashboard.tasks.form.id')">
        <BFormGroup :invalid-feedback="v$.id.$errors[0]?.$message.toString()">
          <label for="idInput">{{ $t('dashboard.tasks.form.id') }}</label>
          <BInput id="idInput" type="text" v-model="v$.id.$model" :state="v$.id.$dirty ? !v$.id.$invalid : null"
            size="sm" />
        </BFormGroup>
        <FormHelp :text="$t('dashboard.tasks.form.idDescription')" />
      </BAccordionItem>
      <!-- Notification Settings -->
      <BAccordionItem :title="$t('dashboard.tasks.form.notificationSettings')">
        <BFormGroup class="mb-5">
          <NotificationSettingsForm v-model="form.notificationSettings" />
          <FormHelp :text="$t('dashboard.tasks.form.notificationSettingsDescription')" />
        </BFormGroup>
      </BAccordionItem>
      <!-- Heartbeats group -->
      <BAccordionItem :title="$t('dashboard.tasks.form.heartbeats')">
        <BFormGroup>
          <label for="idInput">{{ $t('dashboard.tasks.form.heartbeatTimeout') }}</label>
          <div class="d-flex align-items-center gap-1">
            <BInput min="5" id="idInput" type="number" v-model.number="v$.heartbeatTimeoutSeconds.$model"
              :state="v$.heartbeatTimeoutSeconds.$dirty ? !v$.heartbeatTimeoutSeconds.$invalid : null" size="sm"
              style="width: 100px;" />
            <span class="ms-2">{{ $t('dashboard.tasks.form.seconds') }}</span>
          </div>
        </BFormGroup>
        <FormHelp :text="$t('dashboard.tasks.form.heartbeatTimeoutDescription')" />
      </BAccordionItem>
      <BAccordionItem :title="$t('dashboard.tasks.form.metadata')">
        <BFormGroup>
          <DashboardMetadataInput class="mb-3" id="metadata-input" v-model="form.metadata" />
        </BFormGroup>
        <FormHelp :text="$t('dashboard.tasks.form.metadataDescription')" />
      </BAccordionItem>
    </BAccordion>

    <BButton type="submit" class="icon-link" :disabled="v$.$invalid || v$.$pending">
      <Icon name="ph:floppy-disk-back-duotone" aria-hidden />
      {{ $t("dashboard.tasks.form.saveTaskButton") }}
    </BButton>
  </BForm>
</template>

<style lang="scss">
#task-form {
  max-width: var(--bs-breakpoint-lg);

  .accordion-item,
  .accordion-button {
    background-color: transparent;
  }
}
</style>
