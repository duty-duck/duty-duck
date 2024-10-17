<script setup lang="ts">
import { useRoute } from "vue-router";
import type IncidentTimeline from "@/components/incident/Timeline.vue";
import { useNow, useIntervalFn } from "@vueuse/core";

const route = useRoute();
const incidentId = route.params.incidentId as string;
const localePath = useLocalePath();
const repo = useIncidentRepository();
const { locale } = useI18n();

const { userProfile } = useAuthMandatory();
const { data: incidentRes, status, refresh } = await repo.useIncident(incidentId);

const acknowledgeIncidentLoading = ref(false);
const incidentTimeline = ref<InstanceType<typeof IncidentTimeline>>();

const acknowledgeIncident = async () => {
  acknowledgeIncidentLoading.value = true;
  await repo.acknowledgeIncident(incidentId);
  await refresh();
  await incidentTimeline.value?.refresh();
  acknowledgeIncidentLoading.value = false;
}

const acknowledgedByCurrentUser = computed(() => {
  return incidentRes.value?.incident.acknowledgedBy.some(user => user.id === userProfile.value?.id);
});

const now = useNow();
const incidentLengthDuration = computed(() => {
  let endedAt = incidentRes.value?.incident.resolvedAt ? new Date(incidentRes.value.incident.resolvedAt) : now.value;
  const duration =
    endedAt.getTime() -
    new Date(incidentRes.value!.incident.createdAt).getTime();

  return formatDuration(duration, locale.value);
});

useIntervalFn(() => refresh(), 10000);
</script>

<template>
  <div>
    <BContainer>
      <BBreadcrumb>
        <BBreadcrumbItem :to="localePath('/dashboard')">{{
          $t("dashboard.mainSidebar.home")
          }}</BBreadcrumbItem>
        <BBreadcrumbItem :to="localePath('/dashboard/incidents')">{{
          $t("dashboard.mainSidebar.incidents")
          }}</BBreadcrumbItem>
        <BBreadcrumbItem active>{{ $t("dashboard.incidents.incidentDetails")
          }}</BBreadcrumbItem>
      </BBreadcrumb>
      <section class="mb-5">
        <h1 class="mb-4 fs-2">{{ $t("dashboard.incidents.defaultIncidentTitle", { date: $d(new Date(incidentRes!.incident.createdAt), "long") }) }}</h1>
        <div class="d-flex align-items-center gap-2 mb-5">
          <IncidentStatusPill :status="incidentRes!.incident.status" />
          <span class="text-secondary">
            {{ $t('dashboard.incidents.startedAt') }}: {{
              $d(new Date(incidentRes!.incident.createdAt), "long") }}
          </span>
        </div>
        <BCard class="mb-4">
          <IncidentSource :incident-source-id="incidentRes!.incident.incidentSourceId" :incident-source-type="incidentRes!.incident.incidentSourceType" />
        </BCard>
        <div class="row row-gap-3">
          <div class="col-lg-4">
            <BCard class="h-100">
              <p>{{ $t("dashboard.incidents.rootCause") }}</p>
              <p class="fw-semibold">
              <IncidentCause :incident="incidentRes!.incident" />
              </p>
            </BCard>
          </div>
          <div class="col-lg-4">
            <BCard class="h-100">
              <p>{{ $t("dashboard.incidents.startedAt") }}</p>
              <p class="h4">
                {{ $d(new Date(incidentRes!.incident.createdAt), "long") }}
              </p>
            </BCard>
          </div>
          <div class="col-lg-4">
            <BCard class="h-100">
              <p>{{ $t("dashboard.incidents.incidentLength") }}</p>
              <p class="h4">
                {{ incidentLengthDuration }}
              </p>
            </BCard>
          </div>
        </div>


      </section>

      <section class="mb-5">
        <h5>
          <Icon name="ph:users" />
          {{ $t("dashboard.incidents.people") }}

        </h5>
        <div v-if="incidentRes!.incident.acknowledgedBy.length === 0" class="text-secondary mb-3 d-flex align-items-center justify-content-between">
          <template v-if="incidentRes!.incident.acknowledgedBy.length === 0">
            {{ $t("dashboard.incidents.notAcknowledged") }}
          </template>
          <template v-else>
            {{ $t("dashboard.incidents.acknowledgedBy", { count: incidentRes!.incident.acknowledgedBy.length }) }}
          </template>
          <BButton 
            v-if="!acknowledgedByCurrentUser"
            class="icon-link"
            variant="primary"
            @click="acknowledgeIncident"
            :disabled="acknowledgeIncidentLoading"
            pill
          >
            <BSpinner v-if="acknowledgeIncidentLoading" small label="Small spinner" />
            <Icon v-else name="ph:check-bold" />
            {{ $t("dashboard.incidents.acknowledge") }}
          </BButton>
        </div>

        <div class="d-flex gap-1 align-items-center">
          <UserAvatar v-for="user in incidentRes!.incident.acknowledgedBy" :key="user.id" :user="user" showTooltip
            size="2.4rem" fontSize=".9rem" />

        </div>

      </section>
      <IncidentTimeline :incidentId="incidentId" ref="incidentTimeline" />
    </BContainer>
  </div>
</template>
