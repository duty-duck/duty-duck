<script setup lang="ts">
const apiTokensRepository = useApiAuthTokensRepository();
const { data, refresh: refreshTokens } = await apiTokensRepository.useApiAuthTokens();
const showCreateModal = ref(false);

const apiTokensHeadersHelp = 'X-Api-Token-Id: <token-id>\nX-Api-Token-Secret-Key: <token-secret-key>'

const removeToken = async (tokenId: string) => {
    await apiTokensRepository.deleteApiAuthToken(tokenId);
    await refreshTokens();
}
</script>

<template>
    <BCard no-body>
        <BCardBody>
            <div class="d-flex align-items-center justify-content-between mb-3">
                <BCardTitle>{{ $t("dashboard.apiTokens.title") }}</BCardTitle>
                <BButton @click="showCreateModal = true" variant="primary" size="sm" class="d-flex align-items-center gap-2">
                    <Icon name="ph:plus" />
                    {{ $t("dashboard.apiTokens.createToken") }}
                </BButton>
            </div>
            <p>{{ $t("dashboard.apiTokens.description") }}</p>
            <pre id="api-tokens-headers-help">{{ apiTokensHeadersHelp }}</pre>
        </BCardBody>
        <BListGroup flush v-if="data?.apiTokens.length">
            <BListGroupItem v-for="token in data.apiTokens" :key="token.id"
                class="d-flex justify-content-between align-items-start">
                <div class="d-flex flex-column gap-1">
                    {{ token.label }}
                    <div class="text-secondary" style="font-family: monospace;">
                        {{  token.id }}
                    </div>
                    <div class="text-secondary">
                        {{ $t('dashboard.apiTokens.expiresAt', { date: $d(new Date(token.expiresAt!), 'long') }) }}
                    </div>
                </div>
                <div>
                    <BButton size="sm" @click="removeToken(token.id)">
                        {{ $t("dashboard.apiTokens.removeToken") }}
                    </BButton>
                </div>
            </BListGroupItem>
        </BListGroup>
        <div class="text-center my-3" v-else>
            <p>{{ $t("dashboard.apiTokens.noTokens") }}</p>
        </div>
    </BCard>
    <MyAccountCreateApiTokenModal v-model="showCreateModal" @token-created="refreshTokens()" />
</template>

<style scoped lang="scss">
#api-tokens-headers-help {
    background-color: var(--bs-gray-800);
    color: var(--bs-gray-100);
    padding: 1rem;
    border-radius: 0.5rem;
}
</style>
