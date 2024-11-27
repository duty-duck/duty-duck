<script setup lang="ts">
const docPath = useCurrentPath();

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
            <ContentDoc :path="docPath">
                <template v-slot:empty="{ doc }">
                    <h1>{{ doc.title }}</h1>
                </template>
                <template v-slot="{ doc }">
                    <article>
                        <h1>{{ doc.title }}</h1>
                        <ContentRenderer :value="doc" />
                    </article>
                </template>
            </ContentDoc>
        </BContainer>
    </ShowcaseDocumentationLayout>
</template>
