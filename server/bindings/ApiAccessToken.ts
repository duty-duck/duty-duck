// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Permission } from "./Permission";

export type ApiAccessToken = { id: string, organizationId: string, userId: string, label: string, secretKey: Array<number>, scopes: Array<Permission>, expiresAt: string, createdAt: string, };
