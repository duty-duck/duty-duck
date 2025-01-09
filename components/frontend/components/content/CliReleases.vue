<!-- A Component to display the CLI releases, fetched from the API endpoint, which itself fetches the data from the S3 bucket -->
<script setup lang="ts">
import { type Response } from '@/server/routes/api/list-cli-releases';
const { data } = await useFetch<Response>('/api/list-cli-releases');
</script>

<template>
    <BAccordion flush>
        <BAccordionItem v-for="release in data?.releases" :key="release.name" :title="release.name">
            <ul>
                <li v-for="platform in release.platforms" :key="platform.fileName">{{ platform.platform }} - {{
                    platform.version }} -

                    <a :href="platform.url" target="_blank">
                        <Icon name="ph:download" />
                        Download
                    </a>
                </li>
            </ul>
        </BAccordionItem>
    </BAccordion>
</template>
