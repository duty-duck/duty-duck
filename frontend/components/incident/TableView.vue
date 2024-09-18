<script setup lang="ts">
import type { IncidentWithSources } from 'bindings/IncidentWithSources';
const localePath = useLocalePath();

const props = defineProps<{
    incidents: IncidentWithSources[]
}>() 
</script>

<template>
    <BCard no-body class="incidents-table mb-3">
        <BTableSimple hover responsive>
            <BThead>
                <BTr>
                    <BTh>
                        <Icon name="ph:calendar-duotone" aria-hidden /> {{ $t('dashboard.incidents.date') }}
                    </BTh>
                    <BTh>
                        <Icon name="ph:circle-dashed" aria-hidden />
                        {{ $t('dashboard.incidents.status') }}
                    </BTh>
                    <BTh>
                        <Icon name="ph:cylinder" aria-hidden />
                        {{ $t('dashboard.incidents.source') }}
                    </BTh>
                    <BTh>
                        <Icon name="ph:siren" aria-hidden />
                        {{ $t('dashboard.incidents.rootCause') }}
                    </BTh>
                    <BTh>
                        <Icon name="ph:magnifying-glass" aria-hidden />
                    </BTh>
                </BTr>
            </BThead>
            <BTbody>
                <BTr v-for="incident in props.incidents" :key="incident.id" class="my-4">
                    <BTd>
                        <Icon name="ph:clock" aria-hidden /> {{ $t('dashboard.incidents.startedAt') }} {{ $d(new
                            Date(incident.createdAt), "long") }}
                        <div v-if="incident.resolvedAt">

                            <Icon name="ph:check-circle" aria-hidden /> {{ $t('dashboard.incidents.resolvedAt') }}: {{
                                $d(new Date(incident.resolvedAt), "long") }}
                        </div>
                    </BTd>
                    <BTd :class="{ 'text-danger': incident.status == 'ongoing' }">
                        {{ $t(`dashboard.incidentStatus.${incident.status}`) }}
                    </BTd>
                    <BTd>
                        <div v-for="s in incident.sources" :key="s.id">
                            <IncidentCardHttpMonitorDetails :http-monitor-id="s.id" :incident="incident"
                                v-if="s.type == 'HttpMonitor'" />
                        </div>
                    </BTd>
                    <BTd>
                        <IncidentCause :incident="incident" concise />
                    </BTd>
                    <BTd>

                    </BTd>
                </BTr>
            </BTbody>
        </BTableSimple>
    </BCard>
</template>

<style lang="scss" scoped>
.incidents-table {
    padding: 2px;

    .table-responsive {
        margin-bottom: 0;
    }

    table {
        tr:last-child {
            td {
                border-bottom: none
            }
        }

        th {
            color: var(--bs-secondary);
            font-weight: lighter;
        }
    }

}
</style>