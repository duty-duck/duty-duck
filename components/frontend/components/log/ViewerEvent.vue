<script lang="ts">
import type { IndexedTaskRunLogEvent } from 'bindings/IndexedTaskRunLogEvent';
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';


/**
 * A type that can represent a single log event, as rreturned by the task run API, or as returned by the general log api (TODO)
 */
export type LogEvent =
    { eventType: 'taskRun' } & IndexedTaskRunLogEvent | { eventType: 'log' };
</script>

<script setup lang="ts">
const { event, bodyColumns = [], expanded } = defineProps<{ event: LogEvent, bodyColumns: string[], expanded?: boolean }>();

const emits = defineEmits<{
    toggle: []
}>();

const date = computed(() => {
    if (event.eventType == 'taskRun') {
        return new Date(event.timestamp);
    }

    return null;
});

const body = computed(() => {
    if (event.eventType == 'taskRun') {
        return event.body as { [key: string]: any };
    }

    return {};
});

const severityLevelPill = computed(() => {
    if (event.eventType == 'taskRun') {
        if (!event.severityNumber) {
            return null;
        }

        const n = event.severityNumber;
        if (n <= 4) {
            return "TRACE"
        }
        if (n <= 8) {
            return "DEBUG"
        }
        if (n <= 12) {
            // add traling space so all levels have the same width
            return "INFO "
        }
        if (n <= 16) {
            return "WARN"
        }
        if (n <= 20) {
            return "ERROR"
        }
        return "FATAL"

    }

    return null;
});
</script>

<template>
    <div class="event">
        <div class="event-line" @click="emits('toggle')">
            <div class="pill" v-if="date">
                {{ $d(date, "log") }}
            </div>
            <div class="pill" v-if="severityLevelPill">
                {{ severityLevelPill }}
            </div>

            <div class="column-data" v-for="c in bodyColumns">
                <span v-if="body[c]">
                    {{ body[c] }}
                </span>
            </div>
        </div>
        <div class="expanded-section" v-if="expanded">
            <div class="body">
                <VueJsonPretty :data="event" />
            </div>
            <div class="menu">
                <BButton variant="link-primary" @click="emits('toggle')" size="sm">
                    <Icon name="ph:caret-up-thin" />
                    Close
                </BButton>
            </div>
        </div>

    </div>
</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";

.event {
    border-bottom: solid $gray-400 1px;
    box-sizing: border-box;
}

.expanded-section {
    .body {
        overflow: auto;
        padding-bottom: 10px;
        padding-left: 10px;
        height: 240px;
    }

    .menu {
        border-top: solid $gray-300 1px;
        height: 32px;
    }
}

.event-line {
    // layout settings
    height: 36px;
    display: flex;
    align-items: center;
    gap: .3rem;

    // font settings
    @include monospace;
    font-size: 12px;

    // styling
    cursor: pointer;

    &:hover {
        background-color: rgba($color: $primary, $alpha: .075);
    }
}

.column-data {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.pill {
    padding: 2px 4px;
    border-radius: 4px;
    background-color: $gray-200;
    color: $gray-800;
    flex-shrink: 0;
    font-weight: 200;
}
</style>

<style lang="scss">
@import "~/assets/main.scss";

.vjs-value-string {
    color: $primary;
}
</style>