<script setup lang="ts">
    let incidentRepo = useIncidentRepository();
    const localePath = useLocalePath()
    let { refresh: refreshIncidentCount, data: incidentCount } = await incidentRepo.useOngoingIncidentsCount();
</script>

<template>
    <div class="py-2 px-lg-3 px-lg-4 mt-lg-4">
        <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard')">
                    <Icon name="ph:house-simple-duotone" size="20px" />
                    {{ $t("dashboard.sidebar.home") }}
                </NuxtLink>
            </li>
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/monitors')">
                    <Icon name="ph:pulse-duotone" size="22px" />
                    {{ $t("dashboard.sidebar.monitors") }}
                </NuxtLink>
            </li>
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/incidents')">
                    <Icon name="ph:seal-warning-duotone" size="22px" />
                    {{ $t("dashboard.sidebar.incidents") }}
                    <BBadge class="ms-2" variant="danger" v-if="incidentCount && incidentCount > 0">{{ incidentCount }}</BBadge>
                </NuxtLink>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link disabled" href="#">
                    <Icon name="ph:speedometer-duotone" size="22px" />
                    Web perf.
                    <BBadge>{{ $t('dashboard.sidebar.soon') }}</BBadge>
                </a>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link disabled" href="#">
                    <Icon name="ph:cpu-duotone" size="22px" />
                    Infrastructure
                    <BBadge>{{ $t('dashboard.sidebar.soon') }}</BBadge>
                </a>
            </li>
        </ul>
    </div>
</template>