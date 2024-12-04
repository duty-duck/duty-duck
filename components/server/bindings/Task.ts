// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { TaskStatus } from "./TaskStatus";

export type Task = { id: string, organizationId: string, name: string, description: string | null, status: TaskStatus, previousStatus: TaskStatus | null, lastStatusChangeAt: string | null, cronSchedule: string | null, nextDueAt: string | null, startWindowSeconds: number, latenessWindowSeconds: number, heartbeatTimeoutSeconds: number, createdAt: string, };
