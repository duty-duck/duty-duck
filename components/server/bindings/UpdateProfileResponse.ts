// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Permission } from "./Permission";
import type { User } from "./User";

export type UpdateProfileResponse = { needsSessionInvalidation: boolean, newUser: User, newUserPermissions: Array<Permission>, };
