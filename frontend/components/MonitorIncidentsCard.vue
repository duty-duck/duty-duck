<script setup lang="ts">
import type { AsyncDataRequestStatus } from '#app';
import type { HttpMonitor } from 'bindings/HttpMonitor';
import type { IncidentWithSources } from 'bindings/IncidentWithSources';
import type { ListIncidentsResponse } from 'bindings/ListIncidentsResponse';

const { monitor, onGoingIncident, incidents, currentTab } = defineProps<{
    monitor: HttpMonitor,
    onGoingIncident: IncidentWithSources | null,
    incidents: { status: AsyncDataRequestStatus, data: ListIncidentsResponse | null },
    currentTab: "ongoing" | "history",
    incidentsPageNumber: number,
}>();
const emits = defineEmits<{ changePage: [page: number], changeTab: [tab: 'ongoing' | 'history'] }>()

</script>
<template>
    <BCard header-tag="nav" no-body>
        <template #header>
            <BNav card-header tabs>
                <BNavItem @click="emits('changeTab', 'ongoing')" :active="currentTab == 'ongoing' && !!onGoingIncident"
                    :disabled="!onGoingIncident">
                    <Icon v-if="onGoingIncident" name="ph:seal-warning-duotone" class="text-danger" size="1.2rem"
                        style="position: relative; top: .2rem" />
                    {{ $t('dashboard.monitors.ongoingIncident') }}
                </BNavItem>
                <BNavItem @click="emits('changeTab', 'history')" :active="currentTab == 'history' || !onGoingIncident">
                    {{ $t('dashboard.monitors.incidentHistory') }}
                </BNavItem>
            </BNav>
        </template>
        <BCardBody v-if="currentTab == 'ongoing' && onGoingIncident">
            <p>
                <span class="text-secondary">
                    {{ $t('dashboard.incidents.startOfIncident') }}
                </span>
                <br />
                {{ $d(new Date(onGoingIncident.createdAt), 'long') }}
            </p>
            <p>
                <span class="text-secondary">{{ $t('dashboard.incidents.rootCause') }}:</span><br />
                <template v-if="onGoingIncident.cause?.causeType == 'HttpMonitorIncidentCause'">
                    <p> {{ $t('dashboard.httpMonitorIncidents.httpMonitorFailure') }} </p>
                    <p class="lead" v-if="onGoingIncident.cause.errorKind == 'httpcode'">
                        {{ $t('dashboard.httpMonitorIncidents.invalidHttpCode', {
                            httpCode:
                        onGoingIncident.cause.httpCode}) }}
                    </p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'timeout'"> {{
                        $t('dashboard.httpMonitorIncidents.timedOut') }}</p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'redirect'">
                        {{ $t('dashboard.httpMonitorIncidents.tooManyRedirections') }}
                    </p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'connect'">
                        {{ $t('dashboard.httpMonitorIncidents.cannotConnectToEndpoint') }}
                    </p>
                </template>

            </p>
        </BCardBody>
        <template v-else-if="incidents.data">
            <BListGroup flush>
                <BListGroupItem href="#" v-for="i in incidents?.data?.items" :key="i.id">
                    Incident
                </BListGroupItem>
            </BListGroup>

            <BCardBody>
                <BPagination :modelValue="incidentsPageNumber"
                    @update:modelValue="(page: number) => emits('changePage', page)" :prev-text="$t('pagination.prev')"
                    :next-text="$t('pagination.next')" :total-rows="incidents!.data!.totalNumberOfFilteredResults"
                    :per-page="15" />
            </BCardBody>
        </template>
    </BCard>
</template>