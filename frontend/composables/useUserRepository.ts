import { useDebounceFn } from "@vueuse/core";
import { type SignUpCommand } from "bindings/SignUpCommand"
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import type { UpdateProfileResponse } from "bindings/UpdateProfileResponse";
import { FetchError } from "ofetch";

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
        async updateProfile(command: UpdateProfileCommand) {
            return await $fetch<UpdateProfileResponse>("/users/me", { method: "put", body: command })
        }
    }
}