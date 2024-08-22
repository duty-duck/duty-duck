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
          Which URL would you like us to monitor?
        </div>
      </div>
    </div>
    <label class="h5">Reresh interval</label>
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
          The refresh interval determines how often we will check your URL's
          health. A shorter interval enables issues to be detected more quickly,
          at the expense of greater load on your server.
        </div>
      </div>
    </div>
    <div class="row mb-4">
      <div class="col-xl-9">
        <BFormGroup class="mb-2">
          <label for="tags-input" class="h5">Tags</label>
          <BFormTags v-model="form.tags" />
        </BFormGroup>
        <div class="text-secondary">
          <Icon
            style="vertical-align: middle"
            name="ph:question-duotone"
            size="1.5rem"
          />
          Tags are optional. We use this to group monitors, so you can manage
          them more easily.
        </div>
      </div>
    </div>
    <div>
      <BButton type="submit" class="icon-link" :disabled="!formIsComplete">
        <Icon name="ph:floppy-disk-back-duotone" aria-hidden />
        {{  $t('dashboard.monitors.form.save') }}
      </BButton>
    </div>
  </BForm>
</template>
