import { useUrlSearchParams } from "@vueuse/core";
import type { MetadataFilter } from "bindings/MetadataFilter";
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

export const useMetadataFilterQuery = () => {
    const searchParams = useUrlSearchParams<{ [key: string]: string | string[] }>("history");
    return {
        data: computed<MetadataFilter>({
            get() {
                const items: { [key: string]: string[] } = {};
                for (const [key, value] of Object.entries(searchParams)) {
                    if (key.startsWith('meta_')) {
                        if (typeof value === 'string') {
                            items[key.slice(5)] = [value];
                        } else if (Array.isArray(value)) {
                            items[key.slice(5)] = value;
                        }
                    }
                }

                return { items };
            },
            set(filter: MetadataFilter) {
                console.log("Setting metadata filter", filter);
                for (const key in searchParams) {
                    if (key.startsWith('meta_')) {
                        delete searchParams[key];
                    }
                }
                for (const [key, value] of Object.entries(filter.items)) {
                    if (value) {
                        searchParams[`meta_${key}`] = value;
                    }
                }
            }
        }),
        clear() {
            for (const key in searchParams) {
                if (key.startsWith('meta_')) {
                    delete searchParams[key];
                }
            }
        }
    }
}