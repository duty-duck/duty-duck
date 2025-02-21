import { useUrlSearchParams } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { MetadataFilter } from "bindings/MetadataFilter";

export const useDateRangeQuery = () => {
    const dateStart = useRouteQuery<string, Date | null>("dateStart", undefined, { transform: (dateStr?: string) => dateStr ? new Date(dateStr) : null });
    const dateEnd = useRouteQuery<string, Date | null>("dateEnd", undefined, { transform: (dateStr?: string) => dateStr ? new Date(dateStr) : null });

    return computed<{ start: Date, end: Date } | null>({
        get: () => {
            if (dateStart.value && dateEnd.value) {
                return { start: dateStart.value, end: dateEnd.value }
            } else {
                return null
            }
        },
        set: (range: { start: Date, end: Date } | null) => {
            if (range) {
                dateStart.value = range.start;
                dateEnd.value = range.end;
            } else {
                dateStart.value = null;
                dateEnd.value = null;
            }
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