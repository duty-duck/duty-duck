<script lang="ts">
export type TimeRange = "-10m" | "-1h" | "-6h" | "-12h" | "-24h" | "-7d" | "-30d" | null;
export const standardRanges: TimeRange[] = ["-10m", "-1h", "-6h", "-12h", "-24h", "-7d", "-30d"];
</script>

<script lang="ts" setup>
import { OnClickOutside } from '@vueuse/components'

const isOpen = ref(false);

const range = defineModel<TimeRange>({ required: true });
watch(range, () => {
    isOpen.value = false;
});
</script>

<template>
    <OnClickOutside @trigger="isOpen = false">

        <div class="time-range-picker">
            <BButton variant="outline-secondary" @click="isOpen = !isOpen" class="icon-link">
                <Icon name="ph:clock-fill" />
                <template v-if="range === null">
                    {{ $t('dashboard.timeRangeInput.noRange') }}
                </template>
                <template v-else>
                    {{ $t(`dashboard.timeRangeInput.ranges.${range}`) }}
                </template>
            </BButton>
            <div v-if="isOpen" class="time-range-picker-dropdown">
                <BListGroup class="mb-2 shadow-sm">
                    <BListGroupItem :active="range === null" @click="range = null" class="icon-link">
                        <Icon name="ph:x-square-fill" />
                        {{ $t('dashboard.timeRangeInput.noRange') }}
                    </BListGroupItem>
                </BListGroup>
                <BListGroup class="mb-2 shadow-sm">
                    <BListGroupItem :active="range === r" v-for="r in standardRanges" @click="range = r">{{
                        $t(`dashboard.timeRangeInput.ranges.${r}`) }}</BListGroupItem>
                </BListGroup>
            </div>
        </div>
        <Teleport to="body">
            <div id="time-range-picker-backdrop" v-if="isOpen" />
        </Teleport>
    </OnClickOutside>

</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.time-range-picker {
    position: relative;
}

.list-group-item {
    cursor: pointer;
}

.time-range-picker-dropdown {
    position: absolute;
    min-width: 100%;
    top: calc(100% + 8px);
    left: 0;
    z-index: 1000;
}

button {
    min-width: 230px;
    text-align: left;
}

#time-range-picker-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.05);
}
</style>
