<script setup lang="ts">
import type { EntityMetadata } from 'bindings/EntityMetadata';

const { t } = useI18n();

// refs
const newKeyFormRef = ref<HTMLInputElement | null>(null);
const newValueFormRef = ref<HTMLInputElement | null>(null);
const editedValueFormRef = ref<HTMLInputElement | null>(null);

// model
const { readOnly } = defineProps<{
    readOnly?: boolean
}>();
const metadata = defineModel<EntityMetadata>({ required: true });
const newKeyPair = reactive({ key: '', value: '' });
const newPairIsValid = computed(() => newKeyPair.key.trim().length > 0 && newKeyPair.value.trim().length > 0);
const currentlyEditedKey = ref<{ originalKey: string, newKey: string, newValue: string } | null>(null);

const toggleEditMode = (key: string) => {
    currentlyEditedKey.value = { originalKey: key, newKey: key, newValue: metadata.value.records[key] };
}

const addMetadata = () => {
    if (!newPairIsValid.value) return;
    if (!metadata.value || metadata.value.records === undefined) metadata.value = { records: {} };

    metadata.value.records[newKeyPair.key] = newKeyPair.value;
    newKeyPair.key = '';
    newKeyPair.value = '';
    newKeyFormRef.value?.focus();
}

const saveEditedMetadata = () => {
    const { originalKey, newKey, newValue } = currentlyEditedKey.value!;

    if (!newKey.trim() || !newValue.trim()) {
        delete metadata.value.records[originalKey];
    } else if (newKey !== originalKey) {
        delete metadata.value.records[originalKey];
        metadata.value.records[newKey] = newValue;
    } else {
        metadata.value.records[originalKey] = newValue;
    }

    currentlyEditedKey.value = null;
    newKeyFormRef.value?.focus();
}

const sortedRecords = computed(() => Object.fromEntries(Object.entries(metadata.value.records).sort(([a], [b]) => a.localeCompare(b))));
</script>

<template>
    <div id="metadata-input">
        <div v-if="readOnly && Object.keys(metadata.records).length === 0" class="text-muted">
            {{ $t("dashboard.metadataInput.noMetadata") }}
        </div>
        <ul>
            <li class="mb-2" v-for="(value, key) in sortedRecords" :key="key">
                <form class="d-flex gap-1 align-items-center"
                    v-if="!readOnly && currentlyEditedKey?.originalKey === key" @submit.prevent="saveEditedMetadata">
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
                            @keydown.enter.prevent="saveEditedMetadata" />
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
                        <BButton v-if="!readOnly" @click="toggleEditMode(key as string)" variant="link-secondary"
                            size="sm">
                            <Icon name="ph:pencil-bold" />
                        </BButton>
                    </div>
                </div>
            </li>

            <li class=" mt-3" v-if="!readOnly">
                <label class="form-label text-muted">{{ $t('dashboard.metadataInput.newMetadata') }}</label>
                <form class="d-flex gap-1" @submit.prevent="addMetadata">
                    <div class="col">
                        <BInput ref="newKeyFormRef" v-model="newKeyPair.key" :placeholder="t('dashboard.metadataInput.newKey')"
                            size="sm" @keydown.enter.prevent="newValueFormRef?.focus()" />
                    </div>
                    <div class="col">
                        <BInput v-model="newKeyPair.value" :placeholder="t('dashboard.metadataInput.newValue')" size="sm"
                            ref="newValueFormRef" @keydown.enter.prevent="addMetadata" />
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
#metadata-input {
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