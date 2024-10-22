<script setup lang="ts">
import { Body } from '#build/components';
import type { MarkdownRoot } from '@nuxt/content';

const removeTitleFromBody = (body: MarkdownRoot | null) => {
    if (!body) return null
    return body.children.filter(node => node.tag !== 'h1')
}
</script>

<template>
    <ShowcaseLayout>
        <BContainer>
            <main id="blog-post">
                <ContentDoc v-slot="{ doc }">
                    <article>
                        <h1>{{ doc.title }}</h1>
                        <p class="text-muted">
                            {{ $d(new Date(doc.date), 'short') }}
                        </p>
                        <BImg rounded fluid-grow v-if="doc.image" :src="doc.image" alt="Image" class="my-4" />
                        <ContentRenderer :value="doc" />
                    </article>
                </ContentDoc>
            </main>
        </BContainer>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";

#blog-post {
    @extend .mt-5;
    max-width: 768px;
    margin: 0 auto;

    @include media-breakpoint-up(lg) {
        padding-top: 2rem;
    }
}

h1 {
    @include homepage-heading;
}
</style>
