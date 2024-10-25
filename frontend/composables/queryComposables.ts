import type { TimeRange } from "~/components/dashboard/TimeRangePicker.vue";

export const useTimeRangeQuery = () => {
    const route = useRoute();
    return computed<TimeRange | null>({
        get() {
            // no filtering if the timeRange query param is explicitly set to null
            // keep only the last 7 days as default
            return route.query.timeRange == "null" ? null : (route.query.timeRange as TimeRange ?? "-7d");
        },
        set(value: TimeRange) {
            navigateTo({ query: { ...route.query, timeRange: value ?? "null" } });
        }
    })
}