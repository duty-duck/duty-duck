import { useNow } from "@vueuse/core"

/**
 * A comopsable to execute a function on a regular basis. Unlike `useIntervalFn` from vue-use,
 * this function does not use `setInterval`. Instead, it uses `useNow` to schedule tasks based on the current wall-clock time.
 * 
 * Thus, two uses of this composable, in seperate components, with the same interval, should execute their functions at the same time
 * @param fn A function to execute
 * @param intervalMs the interval, in milliseconds, between every execution
 * @returns 
 */
export const useSharedIntervalFn = (fn: () => void, intervalMs: number) => {
    const computeNextExecution = () => {
        const nowMs = new Date().getTime();
        const nextExecutionMs = (nowMs + intervalMs) - (nowMs % intervalMs)
        return new Date(nextExecutionMs)
    }
    const { now, pause, resume } = useNow({ controls: true });
    let nextExecution = computeNextExecution();

    watch(now, (now) => {
        if (now >= nextExecution) {
            nextExecution = computeNextExecution();
            fn();
        }
    });

    return { pause, resume }
}

/**
 * `useSharedIntervalFn`, partially applied with an interval of 6 seconds
 */
export const useDataRefreshInterval = (fn: () => void) => useSharedIntervalFn(fn, 6000);