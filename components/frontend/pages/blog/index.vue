<script lang="ts" setup>
const { locale } = useI18n()
const computeLinkDest = useComputeContentLinkDest();
</script>
<template>
    <ShowcaseLayout>
        <BContainer id="blog-container">
            <div class="row">
                <div class="col-12 col-lg-8">
                    <ContentList :path="`/blog/${locale}`" v-slot="{ list }">
                        <BCard v-for="article in list" :key="article._path" class="mb-3" no-body>
                            <BCardBody>
                                <NuxtLink :to="article._path" class="title">
                                    <h2>{{ article.title }}</h2>
                                </NuxtLink>
                                <p class="text-secondary">
                                    {{ $d(new Date(article.date), 'short') }}
                                </p>
                            </BCardBody>
                            <BCardImg v-if="article.image" :src="article.image" alt="Image" class="rounded-0" />
                            <BCardBody>
                                <p>{{ article.description }}</p>
                                <NuxtLink :to="computeLinkDest({ _path: article._path! })">{{ $t('blog.readMore') }}</NuxtLink>
                            </BCardBody>
                        </BCard>
                    </ContentList>
                </div>
            </div>
        </BContainer>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";

#blog-container {
    padding-top: 2rem;
    margin-bottom: 8rem;

    @include media-breakpoint-up(md) {
        padding-top: 5rem;
    }
}

.title {
    text-decoration: none;
    font-family: 'Poppins', sans-serif;

    h2 {
        color: $gray-800;
        font-weight: 800;
        font-size: 2rem;
    }
}
</style>