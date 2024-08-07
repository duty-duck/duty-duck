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
                    Ongoing incident
                </BNavItem>
                <BNavItem @click="emits('changeTab', 'history')" :active="currentTab == 'history' || !onGoingIncident">Incident
                    History</BNavItem>
            </BNav>
        </template>
        <BCardBody v-if="currentTab == 'ongoing' && onGoingIncident">
            <p>
                <span class="text-secondary">Start of incident:</span><br /> {{ onGoingIncident.createdAt }}
            </p>
            <p>
                <span class="text-secondary">Root cause:</span><br />
                <template v-if="onGoingIncident.cause?.causeType == 'HttpMonitorIncidentCause'">
                    <p>HTTP Monitor failure</p>
                    <p class="lead" v-if="onGoingIncident.cause.errorKind == 'httpcode'"> Endpoint responded with a
                        invalid
                        HTTP Code: {{ onGoingIncident.cause.httpCode }} </p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'timeout'"> Endpoint timed out</p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'redirect'">
                        Endpoint had too many redirections
                    </p>
                    <p class="lead" v-else-if="onGoingIncident.cause.errorKind == 'connect'">
                        Could not connect to endpoint
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
                    @update:modelValue="(page: number) => emits('changePage', page)"
                    :total-rows="incidents!.data!.totalNumberOfFilteredResults" :per-page="15" />
            </BCardBody>
        </template>
    </BCard>
</template>