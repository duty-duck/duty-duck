import { useDebounceFn } from "@vueuse/core";
import type { GetProfileResponse } from "bindings/GetProfileResponse";
import { type SignUpCommand } from "bindings/SignUpCommand"
import type { UpdateProfileCommand } from "bindings/UpdateProfileCommand";
import type { UpdateProfileResponse } from "bindings/UpdateProfileResponse";
import type { VerifyPhoneNumberCommand } from "bindings/VerifyPhoneNumberCommand";
import { FetchError } from "ofetch";

export type UserRepository = ReturnType<typeof useUserRepository>;

export const usePublicUserRepository = () => {
    const { public: { serverUrl } } = useRuntimeConfig();
    const server$Fetch = $fetch.create({ baseURL: serverUrl });

    return {
        async signUp(command: SignUpCommand) {
            try {
                await server$Fetch<void>("/users/signup", { method: "post", body: command })
                return "success"
            } catch (e) {
                if (e instanceof FetchError && e.status == 409)
                    return "conflict"
                return "error"
            }
        },

        async checkPasswordStrength(password: string, firstName: string, lastName: string) {
            const res = await server$Fetch<{ score: number }>("/users/check-password", { method: "post", body: { password, firstName, lastName } });
            return res.score >= 3;
        }
    }
}

export const useUserRepository = async () => {
    const $fetch = await useServer$fetch();

    return {
        async useUserProfile() {
            console.log("[UserRepository::useUserProfile] fetching user profile");
            return await useServerFetch<GetProfileResponse>("/users/me", { retry: 3, retryDelay: 2000 });
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