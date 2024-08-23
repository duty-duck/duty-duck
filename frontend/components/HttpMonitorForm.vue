<script lang="ts" setup>
export type HttpMonitorFormData = {
  url: string;
  intervalSeconds: number;
  tags: string[];
};

const props = withDefaults(defineProps<HttpMonitorFormData>(), {
  url: "",
  intervalSeconds: 60,
  tags: () => [],
});
const emits = defineEmits<{
  submit: [HttpMonitorFormData];
}>();

const form = reactive({ ...props });
const formIsComplete = computed(
  () =>
    form.url &&
    (form.url.startsWith("http://") || form.url.startsWith("https://"))
);
</script>

<template>
  <BForm @submit.prevent="emits('submit', form)" @keypress.enter.prevent="">
    <div class="row mb-4">
      <div class="col-xl-9">
        <BFormGroup class="mb-2">
          <label class="h5" for="urlInput">URL</label>
          <BInput
            id="urlInput"
            type="text"
            v-model="form.url"
            placeholder="https://..."
          />
        </BFormGroup>
        <div class="text-secondary">
          <Icon
            style="vertical-align: middle"
            name="ph:question-duotone"
            size="1.5rem"
          />
          {{ $t("dashboard.monitors.form.urlDescription") }}
        </div>
      </div>
    </div>
    <label class="h5">
      {{ $t("dashboard.monitors.form.refreshInterval") }}
    </label>
    <div class="row mb-4">
      <div class="col-xl-9">
        <MonitorIntervalInput
          :value="form.intervalSeconds"
          class="mb-3"
          @change="(interval) => (form.intervalSeconds = interval.seconds)"
        />
        <div class="text-secondary">
          <Icon
            style="vertical-align: middle"
            name="ph:question-duotone"
            size="1.5rem"
          />
          {{ $t("dashboard.monitors.form.refreshIntervalDescription") }}
        </div>
      </div>
    </div>
    <div class="row mb-4">
      <div class="col-xl-9">
        <BFormGroup class="mb-2">
          <label for="tags-input" class="h5">
            {{ $t("dashboard.monitors.form.tags") }}
          </label>
          <BFormTags v-model="form.tags" />
        </BFormGroup>
        <div class="text-secondary">
          <Icon
            style="vertical-align: middle"
            name="ph:question-duotone"
            size="1.5rem"
          />
          {{ $t("dashboard.monitors.form.tagsDescription") }}
        </div>
      </div>
    </div>
    <div>
      <BButton type="submit" class="icon-link" :disabled="!formIsComplete">
        <Icon name="ph:floppy-disk-back-duotone" aria-hidden />
        {{ $t("dashboard.monitors.form.save") }}
      </BButton>
    </div>
  </BForm>
</template>
