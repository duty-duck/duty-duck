// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { OrderDirection } from "./OrderDirection";
import type { OrderTasksBy } from "./OrderTasksBy";
import type { TaskStatus } from "./TaskStatus";

export type ListTasksParams = { include: Array<TaskStatus> | null, query: string | null, pageNumber: number | null, itemsPerPage: number | null, orderBy: OrderTasksBy | null, orderDirection: OrderDirection | null, metadataFilter: MetadataFilter | null, };
