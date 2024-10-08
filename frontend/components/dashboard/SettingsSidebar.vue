<script setup lang="ts">
    const localePath = useLocalePath()
</script>

<template>
    <div class="py-2 ps-lg-4 pe-lg-2 mt-lg-4">
        <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/myOrg')">
                    <Icon name="ph:users-three-duotone" size="20px" />
                    {{ $t("dashboard.settingsSidebar.myOrg") }}
                </NuxtLink>
            </li>
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard/myAccount')">
                    <Icon name="ph:user" size="20px" />
                    {{ $t("dashboard.settingsSidebar.myAccount") }}
                </NuxtLink>
            </li>
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" :to="localePath('/dashboard')" :disabled="!canReadIncidents">
                    <Icon name="ph:arrow-up-left" size="22px" />
                    {{ $t("dashboard.settingsSidebar.backToDashboard") }}
                </NuxtLink>
            </li>
        </ul>
    </div>
</template>