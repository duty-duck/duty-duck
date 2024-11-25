<script setup lang="ts">
import type { RequestHeaders } from 'bindings/RequestHeaders';

const { t } = useI18n();

// refs
const newKeyFormRef = ref<HTMLInputElement | null>(null);
const newValueFormRef = ref<HTMLInputElement | null>(null);
const editedValueFormRef = ref<HTMLInputElement | null>(null);

// model
const headers = defineModel<RequestHeaders>({ required: true });
const newKeyPair = reactive({ key: '', value: '' });
const newPairIsValid = computed(() => newKeyPair.key.trim().length > 0 && newKeyPair.value.trim().length > 0);
const currentlyEditedKey = ref<{ originalKey: string, newKey: string, newValue: string } | null>(null);

const toggleEditMode = (key: string) => {
    currentlyEditedKey.value = { originalKey: key, newKey: key, newValue: headers.value.headers[key] || '' };
}

const addHeader = () => {
    if (!newPairIsValid.value) return;
    if (!headers.value || headers.value.headers === undefined) headers.value = { headers: {} };

    headers.value.headers[newKeyPair.key] = newKeyPair.value;
    newKeyPair.key = '';
    newKeyPair.value = '';
    newKeyFormRef.value?.focus();
}

const saveEditedHeader = () => {
    const { originalKey, newKey, newValue } = currentlyEditedKey.value!;

    if (!newKey.trim() || !newValue.trim()) {
        delete headers.value.headers[originalKey];
    } else if (newKey !== originalKey) {
        delete headers.value.headers[originalKey];
        headers.value.headers[newKey] = newValue;
    } else {
        headers.value.headers[originalKey] = newValue;
    }

    currentlyEditedKey.value = null;
    newKeyFormRef.value?.focus();
}

const sortedHeaders = computed(() => Object.fromEntries(Object.entries(headers.value.headers).sort(([a], [b]) => a.localeCompare(b))));
</script>

<template>
    <div id="headers-input">
        <ul>
            <li class="mb-2" v-for="(value, key) in sortedHeaders" :key="key">
                <form class="d-flex gap-1 align-items-center"
                    v-if="currentlyEditedKey?.originalKey === key" @submit.prevent="saveEditedHeader">
                    <div class="col-0">
                        <Icon name="ph:dots-three-vertical" />
                    </div>
                    <div class="col">
                        <BInput autofocus v-model="currentlyEditedKey.newKey" size="sm"
                            @keydown.enter="editedValueFormRef?.focus()" />
                    </div>
                    <div class="col">
                        <BInput :ref="r => editedValueFormRef = (r as HTMLInputElement)"
                            v-model="currentlyEditedKey.newValue" size="sm"
                            @keydown.enter.prevent="saveEditedHeader" />
                    </div>
                    <div class="col-1">
                        <BButton type="submit" variant="outline-secondary" size="sm">
                            <Icon name="ph:check-bold" />
                        </BButton>
                    </div>
                </form>
                <div v-else class="d-flex gap-1 align-items-center">
                    <div class="col-0">
                        <Icon name="ph:dots-three-vertical" />
                    </div>
                    <span class="col px-1">{{ key }}</span>
                    <span class="col px-1">{{ value }}</span>
                    <div class="col-1">
                        <BButton @click="toggleEditMode(key as string)" variant="link-secondary" size="sm">
                            <Icon name="ph:pencil-bold" />
                        </BButton>
                    </div>
                </div>
            </li>

            <li class=" mt-3">
                <form class="d-flex gap-1" @submit.prevent="addHeader">
                    <div class="col">
                        <BInput ref="newKeyFormRef" v-model="newKeyPair.key" 
                            :placeholder="t('dashboard.requestHeadersInput.newHeaderName')"
                            size="sm" @keydown.enter.prevent="newValueFormRef?.focus()" />
                    </div>
                    <div class="col">
                        <BInput v-model="newKeyPair.value" 
                            :placeholder="t('dashboard.requestHeadersInput.newHeaderValue')" 
                            size="sm" ref="newValueFormRef" 
                            @keydown.enter.prevent="addHeader" />
                    </div>
                    <div class="col-1">
                        <BButton type="submit" variant="outline-secondary" size="sm" :disabled="!newPairIsValid">
                            <Icon name="ph:plus-bold" />
                        </BButton>
                    </div>
                </form>
            </li>
        </ul>
    </div>
</template>

<style lang="scss" scoped>
#headers-input {
    max-width: 600px;
}

ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

li {
    margin: 0;
    padding: 0;
}
</style>