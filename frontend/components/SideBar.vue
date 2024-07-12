<template>
    <div class="py-2 px-lg-3 px-xl-4 mt-lg-4">
        <ul class="nav nav-pills nav-light nav-fill flex-column gap-2">
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" to="/dashboard">
                    <Icon name="ph:house-simple-duotone" size="20px" />
                    Home
                </NuxtLink>
            </li>
            <li class="nav-item">
                <NuxtLink class="nav-link icon-link" to="/dashboard/monitors">
                    <Icon name="ph:pulse-duotone" size="22px" />
                    Monitors
                </NuxtLink>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link" href="#">
                    <Icon name="ph:seal-warning-duotone" size="22px" />
                    Incidents
                </a>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link" href="#">
                    <Icon name="ph:speedometer-duotone" size="22px" />
                    Web perf.
                </a>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link" href="#">
                    <Icon name="ph:cpu-duotone" size="22px" />
                    Infrastructure
                </a>
            </li>
        </ul>
    </div>
</template>