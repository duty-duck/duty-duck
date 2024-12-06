<script setup lang="ts">
const docPath = useCurrentContentPath();
</script>

<template>
    <ShowcaseLayout>
        <BContainer>
            <main id="blog-post">
                <LazyContentDoc :path="docPath">
                    <template v-slot:empty="{ doc }">
                        <h1>{{ doc.title }}</h1>
                    </template>
                    <template v-slot="{ doc }">
                        <article>
                            <h1>{{ doc.title }}</h1>
                        <p class="text-muted">
                            {{ $d(new Date(doc.date), 'short') }}
                        </p>
                        <BImg rounded fluid-grow v-if="doc.image" :src="doc.image" alt="Image" class="my-4" />
                        <ContentRenderer :value="doc" />
                        </article>
                    </template>
                </LazyContentDoc>
            </main>
        </BContainer>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";

#blog-post {
    @extend .mt-5;
    max-width: 768px;
    margin: 0 auto 8rem auto;

    @include media-breakpoint-up(lg) {
        padding-top: 2rem;
    }
}

h1 {
    @include homepage-heading;
}
</style>
