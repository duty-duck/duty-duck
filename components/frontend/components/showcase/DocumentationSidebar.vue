<script setup lang="ts">
import type { NavItem } from '@nuxt/content';

const { locale } = useI18n()
const route = useRoute();
const localePath = useLocalePath();

const docsQuery = await queryContent('docs', locale.value)

const { data: navigation } = await useAsyncData('docs-navigation', async () => {
    const navigation = await fetchContentNavigation(docsQuery)
    return navigation.find(i => i._path == "/docs")?.children?.find(i => i._path == `/docs/${locale.value}`)?.children
})

const currentPath = useCurrentContentPath();
const computeLinkDest = useComputeContentLinkDest();
</script>

<template>
    <BAccordion flush free>
        <BAccordionItem v-for="item of navigation" :key="item._path" :title="item.title" :visible="currentPath.startsWith(item._path)">
            <BNav vertical>
                <BNavItem v-for="child of item.children" :key="child._path" active>
                    <NuxtLink :to="computeLinkDest(child)">{{ child.title }}</NuxtLink>
                </BNavItem>
            </BNav>
        </BAccordionItem>
        <BAccordionItem :title="$t('documentation.developersDocumentation')" :visible="route.fullPath == localePath('/docs/api')">
            <BNav vertical>
                <BNavItem>
                    <NuxtLink :to="localePath('/docs/api')">{{ $t('documentation.apiReference') }}</NuxtLink>
                </BNavItem>
            </BNav>
        </BAccordionItem>
    </BAccordion>
</template>