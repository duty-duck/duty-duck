import { refDebounced } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { HttpMonitorStatus } from "bindings/HttpMonitorStatus";
import type { ListHttpMonitorsParams } from "bindings/ListHttpMonitorsParams";

/**
 * A composable to retrieve monitors filters from the query string and expose them as [ListMonitorsParams]{@link ListHttpMonitorsParams}.
 * Also exposes a function to clear filters.
 */
export const useHttpMonitorsFilters = async () => {
    const query = useRouteQuery("query", "");
    const queryDebounced = refDebounced(query, 250);
    const pageNumber = useRouteQuery("pageNumber", 1, { transform: Number });
    const includeStatuses = useRouteQuery<HttpMonitorStatus[]>("statuses", ['up', 'down', 'suspicious', 'recovering']);
    const { data: metadataFilter, clear: clearMetadataFilter } = useMetadataFilterQuery();

    const listMonitorsParams = computed<ListHttpMonitorsParams>(() => ({
        pageNumber: pageNumber.value,
        include: includeStatuses.value,
        query: queryDebounced.value,
        itemsPerPage: 10,
        metadataFilter: metadataFilter.value,
    }));

    const clearFilters = () => {
        clearMetadataFilter();
        navigateTo({
            query: { pageNumber: 1, query: "", statuses: [] },
        });
    };

    return {
        listMonitorsParams,
        clearFilters,
        metadataFilter,
        includeStatuses,
        query,
        pageNumber
    }
}