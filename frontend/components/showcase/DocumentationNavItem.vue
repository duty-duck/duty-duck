<script lang="ts" setup>
import type { NavItem } from '@nuxt/content';

const { item } = defineProps<{ item: NavItem }>();
</script>

<template>
    <BNavItem :key="item._path">
        <NuxtLink :to="item._path">{{ item.title }}</NuxtLink>
        <BNav vertical v-if="item.children">
            <ShowcaseDocumentationNavItem v-for="child of item.children" :key="child._path" :item="child" />
        </BNav>
    </BNavItem>
</template>