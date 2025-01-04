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

/**
 * A reusable form validator for task id availability
 * Makes a request to the server to check if the task id is available
 */
export const useTaskIdAvailableValidator = () => {
    const { t } = useI18n();
    const repo = useTasksRepository();
    const checkStrength = useDebounceFn(async (taskId: string) => {
        return await repo.checkTaskIdIsAvailable(taskId);
    }, 500, { rejectOnCancel: true });

    return helpers.withMessage(
        t("dashboard.tasks.form.taskIdNotAvailable"),
        helpers.withAsync(checkStrength)
    );
}


export const isValidCrontabComponent = (min: number, max: number) => (component: string): boolean => {
    const trimmed = component.trim();
    if (trimmed === '*') return true;

    // Check for step values (*/2, 1-10/2)
    if (trimmed.includes('/')) {
        const [range, step] = trimmed.split('/');
        const stepNum = parseInt(step);
        if (isNaN(stepNum) || stepNum < 1) return false;
        if (range === '*') return true;
        return isValidCrontabComponent(min, max)(range); // Recursively validate the range part
    }

    // Check for ranges (1-10)
    if (trimmed.includes('-')) {
        const [start, end] = trimmed.split('-');
        const startNum = parseInt(start);
        const endNum = parseInt(end);
        return !isNaN(startNum) && !isNaN(endNum) && startNum < endNum;
    }

    // Check for lists (1,2,3)
    if (trimmed.includes(',')) {
        return trimmed.split(',').every(val => {
            const num = parseInt(val);
            return !isNaN(num) && num >= 0;
        });
    }

    // Check for single integer
    const num = parseInt(trimmed);
    return !isNaN(num) && num >= min && num <= max;
};


export const isValidCrontab = (cronSchedule: string) => {
    const components = cronSchedule.split(' ');
    if (components.length !== 5) return false;
    const [minute, hour, day, month, dayOfWeek] = components;
    return isValidCrontabComponent(0, 59)(minute) &&
        isValidCrontabComponent(0, 23)(hour) &&
        isValidCrontabComponent(1, 31)(day) &&
        isValidCrontabComponent(1, 12)(month) &&
        isValidCrontabComponent(0, 6)(dayOfWeek);
} 