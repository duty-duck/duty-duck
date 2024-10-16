// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IncidentPriority } from "./IncidentPriority";
import type { IncidentStatus } from "./IncidentStatus";
import type { OrderDirection } from "./OrderDirection";
import type { OrderIncidentsBy } from "./OrderIncidentsBy";

/**
 * Parameters for listing incidents
 */
export type ListIncidentsParams = { pageNumber: number | null, itemsPerPage: number | null, status: Array<IncidentStatus> | null, priority: Array<IncidentPriority> | null, fromDate: string | null, toDate: string | null, orderBy: OrderIncidentsBy | null, orderDirection: OrderDirection | null, };
