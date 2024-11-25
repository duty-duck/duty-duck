<script lang="ts" setup>
import type { NavItem } from '@nuxt/content';

const { item } = defineProps<{ item: NavItem }>();
const localePath = useLocalePath();
const { locale } = useI18n();


const { data } = await useAsyncData(item._path, () => queryContent(item._path).findOne())


const linkDest = computed(() => {
    return localePath(item._path.replace(`/${locale.value}/`, '/'))
})
const showItem = computed(() => {
    return !item._draft && !linkDest.value.endsWith('/docs') && !linkDest.value.endsWith(`/${locale.value}`)
})
const children = computed(() => {
    return item.children?.filter(child => !child._draft && child._path !== item._path)
})
</script>

<template>
    <template v-if="showItem">
        <li class="doc-nav-item">
            <NuxtLink v-if="data" :to="linkDest">{{ item.title }}</NuxtLink>
            <span class="text-muted" v-else>{{ item.title }}</span>
            <ul vertical v-if="item.children">
                <ShowcaseDocumentationNavItem v-for="child in children" :key="child._path" :item="child" />
            </ul>
        </li>
    </template>
    <ul v-else vertical v-if="item.children">
        <ShowcaseDocumentationNavItem v-for="child in children" :key="child._path" :item="child" />
    </ul>
</template>
