<script setup lang="ts">
import EditorJS, { type OutputData } from '@editorjs/editorjs';

// @ts-ignore
import Paragraph from '@editorjs/paragraph';
// @ts-ignore
import Header from '@editorjs/header'; 
import type { CommentPayload } from 'bindings/CommentPayload';
import type { JsonValue } from 'bindings/serde_json/JsonValue';

let editorContainer = ref<HTMLDivElement | null>(null);
let editor: EditorJS | null = null;

const { t } = useI18n();
const { initialValue } = defineProps<{initialValue?: CommentPayload}>()
const emit = defineEmits<{
  submit: [payload: CommentPayload];
}>();

onMounted(() => {
    editor = new EditorJS({
        holder: editorContainer.value!,
        minHeight: 60,
        defaultBlock: 'paragraph',
        data: initialValue?.editorjsData as unknown as OutputData,
        tools: {
            header: {
                class: Header,
                inlineToolbar: false,
                config: {
                    levels: [5],
                    defaultLevel: 5
                }
            },
            paragraph: {
                class: Paragraph,
                inlineToolbar: true
            }
        },
        placeholder: t('dashboard.incidents.timeline.addCommentPlaceholder')
    })
});

onUnmounted(() => {
    editor?.destroy();
});

const submit = async () => {
    const data = await editor!.save();
    emit('submit', {
        editorjsData: data as unknown as JsonValue
    });
}
</script>

<template>
    <BCard no-body id="comment-editor">
        <BCardBody>
            <div id="editorjs" ref="editorContainer" />
        </BCardBody>
        <BCardFooter class="d-flex justify-content-end">
            <BButton size="sm" @click="submit">{{ t('dashboard.incidents.timeline.addCommentButtonLabel') }}</BButton>
        </BCardFooter>
    </BCard>
</template>

<style>
#comment-editor {
    max-width: 600px;
}

.ce-block__content, 
.ce-toolbar__content {
 max-width: 90%; 
}
</style>