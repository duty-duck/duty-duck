import type { UseFetchOptions } from "#app";
import { createSharedComposable, useDebounceFn } from "@vueuse/core";
import type { GetProfileResponse } from "bindings/GetProfileResponse";
import { type SignUpCommand } from "bindings/SignUpCommand"
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import type { UpdateProfileResponse } from "bindings/UpdateProfileResponse";
import type { VerifyPhoneNumberCommand } from "bindings/VerifyPhoneNumberCommand";
import { FetchError, type FetchOptions } from "ofetch";

export type UserRepository = ReturnType<typeof useUserRepository>;

export const useUserRepository = createSharedComposable(() => {
    const $fetch = useServer$fetch();
    const userProfile = ref<GetProfileResponse | "loading" | null>(null);

    return {
        userProfile,
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
        async refreshUserProfile() {
            userProfile.value = "loading";
            userProfile.value = await $fetch<GetProfileResponse>("/users/me", { retry: 3, retryDelay: 1000 })
        },
        async updateProfile(command: UpdateProfileCommand) {
            return await $fetch<UpdateProfileResponse>("/users/me", { method: "put", body: command, retry: 3, retryDelay: 1000 })
        },
        async sendPhoneNumberVerificationCode() {
            return await $fetch<void>("/users/me/send-phone-otp", { method: "post",  retry: 3, retryDelay: 1000 })
        },
        async verifyPhoneNumber(code: string) {
            const command: VerifyPhoneNumberCommand = { code };
            return await $fetch<void>("/users/me/verify-phone-otp", { method: "post", body: command })
        }
    }
})