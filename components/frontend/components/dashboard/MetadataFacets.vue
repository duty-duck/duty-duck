<template>
    <div class="metadata-facets">
        <BAccordion>
            <BAccordionItem v-for="facet in props.metadata.items" :key="facet.key">
                <template #title>
                    <h6 class="mb-0">{{ facet.key }}</h6>
                </template>

                <!-- Search input -->
                <BFormInput v-model="facetSearches[facet.key]" size="sm"
                    :placeholder="$t('dashboard.facets.searchPlaceholder')" class="mb-2" />

                <!-- Value checkboxes -->
                <div class="facet-values">
                    <div v-for="value in getFilteredValues(facet)" :key="value.value" class="form-check">
                        <BFormCheckbox size="sm" :model-value="modelValue.items[facet.key]?.includes(value.value)"
                            @update:model-value="onCheckboxChange(facet.key, value.value)" :name="facet.key">
                            {{ value.value }}
                        </BFormCheckbox>
                    </div>
                </div>
            </BAccordionItem>
        </BAccordion>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { FilterableMetadata } from 'bindings/FilterableMetadata'
import type { FilterableMetadataItem } from 'bindings/FilterableMetadataItem'
import type { MetadataFilter } from 'bindings/MetadataFilter';

const props = defineProps<{
    metadata: FilterableMetadata
}>()

const modelValue = defineModel<MetadataFilter>({
    default: () => ({})
})

// State
const facetSearches = ref<Record<string, string>>({})

// Filter values based on search input
const getFilteredValues = (item: FilterableMetadataItem) => {
    const search = facetSearches.value[item.key]?.toLowerCase() || ''
    return item.distinct_values.filter(v =>
        v.value.toLowerCase().includes(search)
    )
}

const onCheckboxChange = (facetName: string, value: string) => {
    const newSelectedValuesForFacet = modelValue.value.items[facetName]?.includes(value) ?
        modelValue.value.items[facetName]!.filter(v => v !== value) :
        [...(modelValue.value.items[facetName] || []), value]
    
    modelValue.value = {
        items: {
            ...modelValue.value.items,
            [facetName]: newSelectedValuesForFacet
        }
    }
}

// Watch for changes in selection and clean up empty arrays
watch(modelValue, (newValue) => {
    const filtered: Record<string, string[]> = {}
    for (const [key, values] of Object.entries(newValue.items)) {
        if (values && values.length > 0) {
            filtered[key] = values
        }
    }
    if (Object.keys(filtered).length !== Object.keys(newValue).length) {
        modelValue.value.items = filtered
    }
}, { deep: true })
</script>

<style lang="scss">
@import "~/assets/main.scss";

.metadata-facets {
    .facet-values {
        max-height: 250px;
        overflow-y: auto;
    }

    .accordion-item,
    .accordion-button {
        border-radius: 0 !important;
        background-color: var(--bs-gray-100);
    }

    .accordion-body {
        padding: .5rem;
    }

    .form-check {
        padding-left: .9rem;
    }
}
</style>
