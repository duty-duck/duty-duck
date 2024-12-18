import { helpers } from "@vuelidate/validators";
import { useDebounceFn } from "@vueuse/core";

/**
 * A reusable form validator for password strength
 */
export const usePasswordValidator = (firsNameAndLastName: Ref<[string, string]>) => {
    const repo = usePublicUserRepository();
    const checkStrength = useDebounceFn(async (password, firstName, lastName) => {
        return await repo.checkPasswordStrength(password, firstName, lastName);
    }, 500, { rejectOnCancel: true });

    return helpers.withMessage(
        "Your password is too weak",
        helpers.withAsync(
            (input: string) => {
                if (input == '') {
                    return true
                } else {
                    return checkStrength(input, firsNameAndLastName.value[0], firsNameAndLastName.value[1])
                }
            },

            // This validator depends on other fields of the form, and so it
            // must be re-evaluated every time these fields change
            [firsNameAndLastName]
        )
    );
}