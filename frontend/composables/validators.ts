import { helpers } from "@vuelidate/validators";

/**
 * A reusable form validator for password strength
 */
export const usePasswordValidator = (firsNameAndLastName: Ref<[string, string]>) => {
    const userRepo = useUserRepository();
    return helpers.withMessage(
        "Your password is too weak",
        helpers.withAsync(
            (input: string) => {
                if (input == '') {
                    return true
                } else {
                    return userRepo.checkPasswordStrength(input, firsNameAndLastName.value[0], firsNameAndLastName.value[1])
                }
            },
                
            // This validator depends on other fields of the form, and so it
            // must be re-evaluated every time these fields change
            [firsNameAndLastName]
        )
    );
}