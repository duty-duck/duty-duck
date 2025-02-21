<script lang="ts">
export type DateRange = { start: Date, end: Date };
</script>

<script lang="ts" setup>
import { OnClickOutside } from '@vueuse/components'
import VueDatePicker from "@vuepic/vue-datepicker";
import "@vuepic/vue-datepicker/dist/main.css";
import { previousMonday, nextSunday, startOfDay, endOfDay, startOfMonth, endOfMonth } from "date-fns";

const isOpen = ref(false);
const model = defineModel<DateRange | null>();
const { d, t, locale } = useI18n();

const rangeButtons: [string, () => DateRange | null][] = [
    ["dashboard.timeRangeInput.noTimeRange", () => null],
    ["dashboard.timeRangeInput.today", () => {
        const now = new Date();
        return {
            start: startOfDay(now),
            end: endOfDay(now)
        }
    }],
    ["dashboard.timeRangeInput.thisWeek", () => {
        const now = new Date();
        return {
            start: startOfDay(previousMonday(now)),
            end: endOfDay(nextSunday(now))
        }
    }],
    ["dashboard.timeRangeInput.thisMonth", () => {
        const now = new Date();
        return {
            start: startOfDay(startOfMonth(now)),
            end: endOfDay(endOfMonth(now))
        }
    }]
]

const buttonLabel = computed(() => {
    if (model.value) {
        const start = d(model.value.start, "short");
        const end = d(model.value.end, "short");
        if (start == end) {
            return start
        } else {
            return `${start} - ${end}`
        }
    } else {
        return t('dashboard.timeRangeInput.label')
    }
});

const datePickerRange = computed<[Date, Date] | null>({
    get: () => model.value ? [model.value.start, model.value.end] as [Date, Date] : null,
    set: (value: [Date, Date] | null) => {
        if (value == null) {
            model.value = null;
        } else {
            model.value = {
                start: value[0],
                end: value[1]
            };
        };
    }
});
</script>

<template>
    <OnClickOutside @trigger="isOpen = false">
        <div class="time-range-picker">
            <BButton variant="outline-secondary" @click="isOpen = !isOpen" class="icon-link">
                <Icon name="ph:clock-fill" />
                {{ buttonLabel }}
            </BButton>
            <BCard v-if="isOpen" no-body class="time-range-picker-dropdown">
                <div class="d-none d-md-block">
                    <VueDatePicker :locale="locale" range inline v-model:model-value="datePickerRange" utc
                        :ui="{ menu: 'time-range-picker-calendar-menu' }"
                        :select-text="t('dashboard.timeRangeInput.select')" />
                </div>
                <div class="time-range-picker-ranges">
                    <BButton v-for="[label, range] in rangeButtons" size="sm" variant="light" @click="model = range()">
                        <Icon v-if="label == 'dashboard.timeRangeInput.noTimeRange'" name="ph:x-square-fill"
                            aria-hidden />
                        {{ $t(label) }}
                    </BButton>
                </div>
            </BCard>
        </div>
    </OnClickOutside>

</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.time-range-picker {
    position: relative;
}

.time-range-picker-dropdown {
    max-width: 80vw;
    position: absolute;
    top: calc(100%);
    left: 0;
    z-index: 1000;
    display: flex;
    flex-direction: row;
}

.time-range-picker-ranges {
    flex-grow: 1;
    padding: .5rem;
    display: flex;
    flex-direction: column;
    gap: .25rem;
}

:global(.time-range-picker-calendar) {
    border: none;
}

button {
    min-width: 180px;
    text-align: left;
}
</style>

<style lang="scss">
@import "~/assets/main.scss";

.time-range-picker-calendar-menu {
    border-right: 1px solid $gray-300;
    border-radius: 0;
    border-top: none;
    border-bottom: none;
    border-left: none;
}
</style>