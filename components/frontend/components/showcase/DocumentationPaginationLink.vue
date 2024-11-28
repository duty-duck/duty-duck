<script setup lang="ts">
const { link, direction = 'prev' } = defineProps<{
    link: NavigationLink,
    direction: 'prev' | 'next'
}>();

const computeLinkDest = useComputeDocumentationLinkDest();
</script>

<template>
    <NuxtLink v-if="link" :to="computeLinkDest(link)" class="d-flex align-items-center gap-3 text-decoration-none border border-muted rounded-2 p-3"
        :class="{ 'flex-row-reverse': direction === 'next' }">
        <Icon :name="direction === 'prev' ? 'ph:arrow-left-bold' : 'ph:arrow-right-bold'" />
        <div class="d-flex flex-column">
            <small class="text-muted">{{ $t(`documentation.${direction}`) }}</small>
            {{ link.title }}
        </div>
    </NuxtLink>
</template>