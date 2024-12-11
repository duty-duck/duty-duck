<script setup lang="ts">
const docPath = useCurrentContentPath();

const { data: nextAndPrevious } = await useNextAndPrevious();

definePageMeta({
    middleware: (to, from) => {
        if (from.fullPath.startsWith("/docs") && to.fullPath.startsWith("/en/docs")) {
            return navigateTo("/en/docs")
        } else if (from.fullPath.startsWith("/en/docs") && to.fullPath.startsWith("/docs")) {
            return navigateTo("/docs")
        }
    }
})
</script>

<template>
    <ShowcaseDocumentationLayout>
        <BContainer class="mt-5">
            <BButton variant="outline-primary" data-bs-toggle="offcanvas" data-bs-target="#docs-offcanvas"
                aria-controls="docs-offcanvas" aria-expanded="false" aria-label="Toggle navigation"
                class="icon-link mb-4 d-lg-none">
                <Icon name="ph:list-bold" />
                Menu
            </BButton>
            <LazyContentDoc :path="docPath">
                <template v-slot:empty="{ doc }">
                    <h1>{{ doc.title }}</h1>
                </template>
                <template v-slot="{ doc }">
                    <article id="doc-content">
                        <h1>{{ doc.title }}</h1>
                        <ContentRenderer :value="doc" />
                    </article>
                </template>
            </LazyContentDoc>

            <div class="d-flex justify-content-between mt-5 mb-4">
                <ShowcaseDocumentationPaginationLink v-if="nextAndPrevious?.prev" :link="nextAndPrevious.prev"
                    direction="prev" />
                <div v-else></div>
                <ShowcaseDocumentationPaginationLink v-if="nextAndPrevious?.next" :link="nextAndPrevious.next"
                    direction="next" />
            </div>
        </BContainer>
    </ShowcaseDocumentationLayout>
</template>

<style lang="scss">
@import "~/assets/main.scss";

#doc-content {
    h1>a,
    h2>a,
    h3>a,
    h4>a,
    h5>a,
    h6>a {
        color: $gray-800;
        text-decoration: none;
    }

    h2,h3,h4,h5,h6 {
        @extend .mt-4;
    }
}
</style>