<template>
    <div class="py-2 px-lg-3 px-xl-4 mt-lg-4">
        <ul class="nav nav-pills nav-fill flex-column gap-2">
            <li class="nav-item">
                <RouterLink class="nav-link icon-link" to="/dashboard">
                    <i data-feather="home"></i>
                    Home
                </RouterLink>
            </li>
            <li class="nav-item">
                <RouterLink class="nav-link icon-link" to="/dashboard/monitors">
                    <i data-feather="eye"></i>
                    Monitors
                </RouterLink>
            </li>
            <li class="nav-item">
                <a class="nav-link icon-link" href="#">
                    <i data-feather="alert-octagon"></i>
                    Incidents
                </a>
            </li>
        </ul>
    </div>
</template>