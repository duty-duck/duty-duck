<script lang="ts">
export const getIncidentLabel = (cause: HttpMonitorIncidentCause, t: ReturnType<typeof useI18n>['t']): string => {
  switch (cause.errorKind) {
    case 'httpcode':
      return t("dashboard.httpMonitorIncidents.invalidHttpCode", {
        httpCode: cause.httpCode,
      });
    case 'timeout':
      return t("dashboard.httpMonitorIncidents.timedOut");
    case 'redirect':
      return t("dashboard.httpMonitorIncidents.tooManyRedirections");
    case 'connect':
      return t("dashboard.httpMonitorIncidents.cannotConnectToEndpoint");
    case 'request':
      return t("dashboard.httpMonitorIncidents.requestError");
    default:
      return t("dashboard.httpMonitorIncidents.unknownError");
  }
};
</script>

<script lang="ts" setup>
import type { HttpMonitorIncidentCause } from "bindings/HttpMonitorIncidentCause";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const { cause } = defineProps<{ cause: HttpMonitorIncidentCause }>();
</script>

<template>
  <span>{{ getIncidentLabel(cause, t) }}</span>
</template>
