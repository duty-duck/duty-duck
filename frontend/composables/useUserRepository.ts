import { useDebounceFn } from "@vueuse/core";
import type { GetProfileResponse } from "bindings/GetProfileResponse";
import { type SignUpCommand } from "bindings/SignUpCommand"
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import type { UpdateProfileResponse } from "bindings/UpdateProfileResponse";
import type { VerifyPhoneNumberCommand } from "bindings/VerifyPhoneNumberCommand";
import { FetchError } from "ofetch";

export type UserRepository = ReturnType<typeof useUserRepository>;

export const useUserRepository = async () => {
    const $fetch = await useServer$fetch();

    return {
        async useUserProfile() {
            console.log("[UserRepository::useUserProfile] fetching user profile");
            return await useServerFetch<GetProfileResponse>("/users/me", { retry: 3, retryDelay: 2000 });
        },
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
            return await $fetch<UpdateProfileResponse>("/users/me", { method: "put", body: command, retry: 3, retryDelay: 1000 })
        },
        async sendPhoneNumberVerificationCode() {
            return await $fetch<void>("/users/me/send-phone-otp", { method: "post", retry: 3, retryDelay: 1000 })
        },
        async verifyPhoneNumber(code: string) {
            const command: VerifyPhoneNumberCommand = { code };
            return await $fetch<void>("/users/me/verify-phone-otp", { method: "post", body: command })
        }
    }
}