// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IncidentEventPayload } from "./IncidentEventPayload";
import type { IncidentEventType } from "./IncidentEventType";

export type IncidentEvent = { organizationId: string, incidentId: string, createdAt: string, eventType: IncidentEventType, eventPayload: IncidentEventPayload | null, };
