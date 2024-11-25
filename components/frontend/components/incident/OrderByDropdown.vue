<script setup lang="ts">
import type { OrderDirection } from 'bindings/OrderDirection';
import type { OrderIncidentsBy } from 'bindings/OrderIncidentsBy';

const orderBy = defineModel<OrderIncidentsBy>("orderBy", { required: true });
const orderDirection = defineModel<OrderDirection>("orderDirection", { required: true });

const { t } = useI18n();
const options: { value: [OrderIncidentsBy, OrderDirection], label: string }[] = [
    { value: ['createdAt', 'desc'], label: t('dashboard.incidentOrderBy.createdAtDesc') },
    { value: ['createdAt', 'asc'], label: t('dashboard.incidentOrderBy.createdAtAsc') },
]
</script>

<template>
    <BDropdown variant="outline-secondary" toggle-class="d-flex align-items-center gap-1">
        <template #button-content>
            <Icon name="ph:sort-ascending" size="1.3rem" />
            {{ $t('dashboard.incidentOrderBy.buttonLabel') }}
        </template>
        <BDropdownItem v-for="option in options" @click="orderBy = option.value[0]; orderDirection = option.value[1]"
            :active="orderBy === option.value[0] && orderDirection === option.value[1]">
            <label style="width: 100%; text-transform: capitalize;" class="ps-2">{{ option.label }}</label>
        </BDropdownItem>
    </BDropdown>
</template>