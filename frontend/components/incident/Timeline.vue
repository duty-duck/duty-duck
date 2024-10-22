<script setup lang="ts">
import { useInfiniteScroll } from '@vueuse/core';
import type { CommentIncidentRequest } from 'bindings/CommentIncidentRequest';
import type { CommentPayload } from 'bindings/CommentPayload';
import type { IncidentEvent } from 'bindings/IncidentEvent';
import type { TimelineItem } from 'bindings/TimelineItem';

const { incidentId, showCommentEditor = true, infiniteScrolling = true } = defineProps<{
    incidentId: string,
    showCommentEditor?: boolean,
    infiniteScrolling?: boolean
}>();
const isCommentLoading = ref(false);
const pageSize = 10;
const state = reactive({ pages: [] as TimelineItem[][], lastIncompleteResponse: null as null | Date, isLoading: false });
const repo = useIncidentRepository();

/**
 * Attempts to load the next page of the incident timeline.
 * If the response is empty, the lastEmptyResponse is set to the current date. No more HTTP calls will be made for 2 minutes.
 */
const loadNextPage = async ({ force = false }: { force?: boolean } = {}) => {
    state.isLoading = true;
    if (!force && state.lastIncompleteResponse && (new Date().getTime() - state.lastIncompleteResponse.getTime()) < 120000) {
        state.isLoading = false;
        return;
    }

    const lastPage = state.pages[state.pages.length - 1];
    const lastPageNumber = state.pages.length;
    const lastPageIsComplete = !!lastPage && lastPage.length >= pageSize;
    // if not page is loaded, we fetch the first page
    // if the last page is not complete, we fetch this page again
    // if it is complete, we fetch the next page
    const pageNumber = state.pages.length > 0 ? (lastPageIsComplete ? lastPageNumber + 1 : lastPageNumber) : 1;
    const response = await repo.getIncidentTimeline(incidentId, {
        pageNumber,
        itemsPerPage: pageSize
    });

    if (!!state.pages[pageNumber - 1]) {
        state.pages[pageNumber - 1] = response.items;
    } else if (response.items.length > 0) {
        state.pages.push(response.items);
    }

    if (response.items.length < pageSize) {
        state.lastIncompleteResponse = new Date();
    } else {
        state.lastIncompleteResponse = null;
    }
    state.isLoading = false;
}

useInfiniteScroll(window, () => {
    loadNextPage();
});

onMounted(loadNextPage);

const addComment = async (payload: CommentPayload) => {
    isCommentLoading.value = true;
    const request: CommentIncidentRequest = { payload };
    await repo.commentIncident(incidentId, request);
    await loadNextPage({ force: true });
    isCommentLoading.value = false;
}

defineExpose({
    refresh: async () => {
        await loadNextPage({ force: true });
    }
});
</script>

<template>
    <section>
        <h5 class="mb-5">{{ $t("dashboard.incidents.timeline.sectionTitle") }}</h5>
        <template v-for="(page, index) in state.pages" :key="`page-${index}`">
            <IncidentTimelineItem v-for="item in page" :key="`${item.event.incidentId}-${item.event.createdAt}`"
                :item="item" />
        </template>

        <IncidentTimelineItem v-if="state.isLoading">
            <BSpinner />
        </IncidentTimelineItem>
        <IncidentTimelineItem v-if="showCommentEditor">
            <label class="text-secondary mb-3">{{ $t("dashboard.incidents.timeline.addComment") }}</label>
            <LazyDashboardCommentEditor @submit="addComment" v-if="!isCommentLoading" />
            <div v-else>
                <BSpinner />
            </div>
        </IncidentTimelineItem>
    </section>
</template>
