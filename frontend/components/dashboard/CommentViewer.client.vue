<script setup lang="ts">
import type { OutputData } from '@editorjs/editorjs';
import type { CommentPayload } from 'bindings/CommentPayload';

const blockComponents = {
    header: resolveComponent('DashboardCommentViewerHeading'),
    paragraph: resolveComponent('DashboardCommentViewerParagraph'),
}

const { comment } = defineProps<{
    comment: CommentPayload
}>();

const blocks = computed(() => {
    const data = comment.editorjsData as unknown as OutputData;
    return data.blocks.map(block => ({
        id: block.id,
        component: blockComponents[block.type as keyof typeof blockComponents],
        data: block.data,
    }))
});
</script>

<template>
    <BCard class="comment">
        <component v-for="block in blocks" :key="block.id" :is="block.component" :data="block.data" />
    </BCard>
</template>

<style scoped lang="scss">
.comment {
    max-width: 600px;
}
</style>