<script lang="ts" setup>
import type { Incident } from "bindings/Incident";
import type { IncidentCause } from "bindings/IncidentCause";

const { incident } = defineProps<{ incident: Incident }>();
</script>
<template>
  <span v-if="incident.cause?.errorKind == 'httpcode'">
    {{
      $t("dashboard.httpMonitorIncidents.invalidHttpCode", {
        httpCode: incident.cause.httpCode,
      })
    }}
  </span>
  <span v-else-if="incident.cause?.errorKind == 'timeout'">
    {{ $t("dashboard.httpMonitorIncidents.timedOut") }}
  </span>
  <span v-else-if="incident.cause?.errorKind == 'redirect'">
    {{ $t("dashboard.httpMonitorIncidents.tooManyRedirections") }}
  </span>
  <span v-else-if="incident.cause?.errorKind == 'connect'">
    {{ $t("dashboard.httpMonitorIncidents.cannotConnectToEndpoint") }}
  </span>
</template>
