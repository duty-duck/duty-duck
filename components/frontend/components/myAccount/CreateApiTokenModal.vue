<script setup lang="ts">
import type { CreateApiTokenRequest } from 'bindings/CreateApiTokenRequest';
import type { CreateApiTokenResponse } from 'bindings/CreateApiTokenResponse';
import type { Permission } from 'bindings/Permission';
import { useClipboard } from '@vueuse/core';

const showModal = defineModel<boolean>();
const { copy, isSupported } = useClipboard()


const emit = defineEmits<{
    'update:modelValue': [value: boolean]
    'token-created': []
}>();

const apiTokensRepository = useApiAuthTokensRepository();
const newToken = ref<CreateApiTokenResponse | null | "failed">(null);

const form = reactive<CreateApiTokenRequest>({
    label: '',
    expiresAt: '',
    scopes: [] as Permission[]
});

async function createToken() {
    try {
        const response = await apiTokensRepository.createApiAuthToken({
            ...form,
            label: form.label.trim(),
            expiresAt: new Date(form.expiresAt).toISOString()
        });
        newToken.value = response;
        emit('token-created');
    } catch (error) {
        console.error('Failed to create token:', error);
        newToken.value = "failed";
    }
}

function closeModal() {
    emit('update:modelValue', false);
    newToken.value = null;
    form.label = '';
    form.expiresAt = '';
    form.scopes = [];
}
</script>

<template>
    <BModal v-model="showModal" :title="$t('dashboard.apiTokens.createToken')" @hide="closeModal">
        <template v-if="newToken === 'failed'"  >
            <BAlert variant="danger" class="mb-3" :model-value="true">
                {{ $t('dashboard.apiTokens.createTokenFailed') }}
            </BAlert>
        </template>

        <template v-else-if="!newToken">
            <BForm @submit.prevent="createToken" class="d-flex flex-column gap-3">
                <p>{{ $t('dashboard.apiTokens.form.description') }}</p>
                <BFormGroup :label="$t('dashboard.apiTokens.form.label')">
                    <BFormInput v-model="form.label" required />
                </BFormGroup>

                <BFormGroup :label="$t('dashboard.apiTokens.form.expiresAt')">
                    <BFormInput v-model="form.expiresAt" type="date" required />
                </BFormGroup>

                <!-- Add scope selection here based on your Permission enum -->
            </BForm>
        </template>

        <template v-else>
            <div class="token-created">
                <p class="mb-3">{{ $t('dashboard.apiTokens.tokenCreated') }}</p>
                <BAlert variant="warning" class="mb-3" :model-value="true">
                    {{ $t('dashboard.apiTokens.secretKeyWarning') }}
                </BAlert>
                <div class="d-flex flex-column gap-2">
                    <div>
                        <strong>{{ $t('dashboard.apiTokens.tokenId') }}:</strong>
                        <div class="token">
                            {{ newToken.id }}
                        </div>
                    </div>

                    <div>
                        <strong>{{ $t('dashboard.apiTokens.secretKey') }}:</strong>
                        <div class="token"> {{ newToken.secretKey }}</div>
                    </div>
                    <div class="d-flex align-items-center gap-2">
                        <BButton size="sm" @click="copy(newToken.secretKey)" v-if="isSupported">
                            <Icon name="ph:copy" />
                            {{ $t('dashboard.apiTokens.copySecretKey') }}
                        </BButton>
                    </div>
                </div>
            </div>
        </template>

        <template #footer>
            <div v-if="!newToken" class="d-flex gap-2">
                <BButton variant="secondary" @click="closeModal">
                    {{ $t('cancel') }}
                </BButton>
                <BButton variant="primary" @click="createToken">
                    {{ $t('create') }}
                </BButton>
            </div>
            <div v-else>
                <BButton variant="primary" @click="closeModal">
                    {{ $t('close') }}
                </BButton>
            </div>
        </template>
    </BModal>
</template>

<style scoped lang="scss">
.token-info {
    background-color: var(--bs-gray-100);
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 1rem;
}

.token {
    font-family: monospace;
    overflow-x: auto;
}
</style>
