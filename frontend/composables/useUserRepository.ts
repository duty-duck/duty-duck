import type { UseFetchOptions } from "#app";
import { useDebounceFn } from "@vueuse/core";
import type { GetProfileResponse } from "bindings/GetProfileResponse";
import { type SignUpCommand } from "bindings/SignUpCommand"
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import type { UpdateProfileResponse } from "bindings/UpdateProfileResponse";
import { FetchError, type FetchOptions } from "ofetch";

export const useUserRepository = () => {
    const $fetch = useServer$fetch();

    return {
        checkPasswordStrength: useDebounceFn(async (password, firstName, lastName) => {
            const res = await $fetch<{ score: number }>("/users/check-password", {
                method: "post",
                body: {
                    password,
                    firstName,
                    lastName
                },
            });

            return res.score >= 3;
        }, 500, { rejectOnCancel: true }),
        async signUp(command: SignUpCommand) {
            try {
                await $fetch<void>("/users/signup", { method: "post", body: command })
                return "success"
            } catch (e) {
                if (e instanceof FetchError && e.status == 409)
                    return "conflict"
                return "error"
            }
        },
        fetchUserProfile() {
            return $fetch<GetProfileResponse>("/users/me", { retry: 3, retryDelay: 1000 })
        },
        async updateProfile(command: UpdateProfileCommand) {
            return await $fetch<UpdateProfileResponse>("/users/me", { method: "put", body: command, retry: 3, retryDelay: 1000 })
        }
    }
}