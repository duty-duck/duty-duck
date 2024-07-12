<script setup lang="ts">
const secondsInterval = [10, 30].map((seconds) => ({
  seconds,
  label: `${seconds} seconds`,
  shortLabel: `${seconds}s`,
}));
const minutesInterval = [1, 5, 10, 30].map((minutes) => ({
  seconds: minutes * 60,
  label: `${minutes} ${minutes == 1 ? "minute" : "minutes"}`,
  shortLabel: `${minutes}m`,
}));
const hoursInterval = [1, 2, 6, 12, 24].map((hours) => ({
  seconds: hours * 3600,
  label: `${hours} ${hours == 1 ? "hour" : "hours"}`,
  shortLabel: `${hours}h`,
}));
const intervals = [...secondsInterval, ...minutesInterval, ...hoursInterval];
const props = defineProps<{ value: number }>();
const index = ref(intervals.findIndex(int => int.seconds == props.value));
const currentInterval = computed(() => intervals[index.value]);

const emit = defineEmits<{
  change: [interval: { seconds: number }];
}>();

watch(currentInterval, (interval) => emit("change", interval));
onMounted(() => emit("change", currentInterval.value));
</script>

<template>
  <div class="px-5 mt-4">
    <div class="position-relative" style="height: 3.25rem">
      <div class="indicator" :style="{ left: `${index * 10}%` }">
        {{ currentInterval.label }}
      </div>
    </div>
    <BFormInput
      v-model="index"
      type="range"
      min="0"
      :max="intervals.length - 1"
      step="1"
    />
    <div class="d-flex justify-content-between text-secondary">
      <span>{{ intervals[0].shortLabel }}</span>
      <span>{{ intervals[4].shortLabel }}</span>
      <span>{{ intervals[10].shortLabel }}</span>
    </div>
  </div>
</template>

<style scoped lang="scss">
.indicator {
  white-space: nowrap;
  background: white;
  position: absolute;
  transform: translateX(-50%);
  padding: 0.5rem 1rem;
  filter: drop-shadow(1px 1px 6px rgba(0, 0, 0, 0.1));
  border-radius: 5px;

  &::after {
    position: absolute;
    bottom: -7px;
    left: calc(50% - 5px);
    content: "";
    width: 0;
    height: 0;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-top: 7px solid white;
  }
}
</style>
