<template>
    <BCard class="ping-event-card" no-body>
        <BTabs card>
            <BTab :title="$t('dashboard.incidents.timeline.pingEvent.tabs.summary')">
                <div class="error-details">
                    <p class="mb-2" v-if="event.errorKind !== 'none'">
                        <strong>{{ $t('dashboard.incidents.timeline.pingEvent.errorKind') }}:</strong>
                        {{ getIncidentLabel({ errorKind: event.errorKind, httpCode: event.httpCode }, t) }}
                    </p>

                    <p v-if="event.httpCode" class="mb-2">
                        <strong>{{ $t('dashboard.incidents.timeline.pingEvent.httpCode') }}:</strong>
                        {{ event.httpCode }}
                    </p>

                    <div v-if="event.resolvedIpAddresses.length" class="mb-2">
                        <strong>{{ $t('dashboard.incidents.timeline.pingEvent.resolvedIps') }}:</strong>
                        <ul class="mt-1 mb-0">
                            <li v-for="ip in event.resolvedIpAddresses" :key="ip">
                                {{ ip }}
                                <span v-if="ip === event.responseIpAddress" class="text-muted">({{
                                    $t('dashboard.incidents.timeline.pingEvent.responseIp') }})</span>
                            </li>
                        </ul>
                    </div>

                    <p class="mb-0" v-if="event.responseTimeMs && Number(event.responseTimeMs) > 0">
                        <strong>{{ $t('dashboard.incidents.timeline.pingEvent.responseTime') }}:</strong>
                        {{ Number(event.responseTimeMs) }}ms
                    </p>
                </div>
            </BTab>
            <BTab v-if="Object.keys(event.httpHeaders).length"
                :title="$t('dashboard.incidents.timeline.pingEvent.tabs.headers')">
                <BTable :items="headerItems" :fields="headerFields" small striped class="my-0" />
            </BTab>
        </BTabs>
    </BCard>
</template>

<script setup lang="ts">
import type { PingEventPayload } from 'bindings/PingEventPayload';
import { getIncidentLabel } from '../httpMonitor/IncidentLabel.vue';

const { t } = useI18n();

const props = defineProps<{
    event: PingEventPayload
}>()

const headerItems = computed(() => {
    return Object.entries(props.event.httpHeaders).map(([key, value]) => ({
        header: key,
        value: value || ''
    }))
})

const headerFields = [
    { key: 'header', label: 'Header' },
    { key: 'value', label: 'Value' }
]
</script>

<style scoped>
.ping-event-card {
    max-width: 600px;
}

.error-details {
    font-size: 0.95rem;
}
</style>