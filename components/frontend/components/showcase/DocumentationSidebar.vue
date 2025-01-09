<script setup lang="ts">
import type { NavItem } from '@nuxt/content';

const { locale } = useI18n()
const route = useRoute();
const { public: { serverUrl } } = useRuntimeConfig();
const localePath = useLocalePath();

const docsQuery = queryContent('docs', locale.value);
const { data: navigation } = await useAsyncData('docs-navigation', async () => {
    const navigation = await fetchContentNavigation(docsQuery)
    return navigation.find(i => i._path == "/docs")?.children?.find(i => i._path == `/docs/${locale.value}`)?.children
})

const currentPath = useCurrentContentPath();
const computeLinkDest = useComputeContentLinkDest();
</script>

<template>
    <BAccordion flush free>
        <BAccordionItem v-for="item of navigation" :key="item._path" :title="item.sectionTitle || item.title" :visible="currentPath.startsWith(item._path)" body-class="px-0">
            <BNav vertical small>
                <BNavItem v-for="child of item.children" :key="child._path">
                    <NuxtLink :to="computeLinkDest(child)">{{ child.title }}</NuxtLink>
                </BNavItem>
                <BNavItem v-if="item._path.endsWith('/developers')">
                    <NuxtLink :to="localePath('/docs/api')">{{ $t('documentation.apiReference') }}</NuxtLink>
                </BNavItem>
                <BNavItem v-if="item._path.endsWith('/developers')">
                    <NuxtLink :to="`${serverUrl}/openapi`" target="_blank">
                        <Icon name="ph:download-duotone" />
                        {{ $t('documentation.apiReference') }} (OpenAPI)
                    </NuxtLink>
                </BNavItem>
            </BNav>

        </BAccordionItem>
    </BAccordion>
</template>