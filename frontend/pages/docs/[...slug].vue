<script setup lang="ts">
import { createReusableTemplate } from '@vueuse/core';

const { locale } = useI18n()
const route = useRoute()
const docPath = computed(() => {
    const slug: string[] = route.params.slug as string[];
    return ["docs", locale.value, ...slug].join('/')
})
const docsQuery = queryContent('docs', locale.value)
const [DefineNavigation, ReuseNavigation] = createReusableTemplate()
</script>

<template>
    <DefineNavigation>
        <NuxtLink to="/docs/api">API Docs</NuxtLink>
        <ContentNavigation v-slot="{ navigation }" :query="docsQuery">
            <BNav vertical>
                <ShowcaseDocumentationNavItem v-for="item of navigation" :key="item._path" :item="item" />
            </BNav>
        </ContentNavigation>
    </DefineNavigation>
    <ShowcaseLayout>
        <div class="offcanvas offcanvas-start" tabindex="-1" id="docs-offcanvas">
            <div class="offcanvas-header">
                <button type="button" class="btn-close" data-bs-dismiss="offcanvas" aria-label="Close"></button>
            </div>
            <div class="offcanvas-body">
                <ReuseNavigation />
            </div>
        </div>
        <div id="docs-container">
            <div id="docs-nav">
                <ReuseNavigation />
            </div>
            <div id="docs-content">
                <BContainer>
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
            </div>
        </div>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import '~/assets/main.scss';

#docs-container {
    display: flex;
    min-height: calc(100vh - $navbar-height);
}

#docs-nav {
    display: none;
    background-color: white;
    z-index: 0;
    border-right: 1px solid var(--bs-gray-200);
    overflow-y: auto;
    padding-top: 1rem;
    min-width: 250px;

    @include media-breakpoint-up(lg) {
        display: block;
    }
}

#docs-content {
    padding-top: 3rem;
    flex: 1;
}
</style>
