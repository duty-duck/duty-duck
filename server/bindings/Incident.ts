// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IncidentCause } from "./IncidentCause";
import type { IncidentPriority } from "./IncidentPriority";
import type { IncidentStatus } from "./IncidentStatus";

/**
 * The base struct used by all incident types
 */
export type Incident = { organizationId: string, id: string, createdAt: string, createdBy: string | null, resolvedAt: string | null, cause: IncidentCause | null, status: IncidentStatus, priority: IncidentPriority, };
