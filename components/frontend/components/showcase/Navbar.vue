<script setup lang="ts">
const localePath = useLocalePath();
const { locale, locales } = useI18n()
const switchLocalePath = useSwitchLocalePath()
</script>

<template>
    <div>
        <BNavbar id="settings-bar">
            <BNavbarNav class="ms-auto text-muted">
                <BNavItemDropdown :text="`${$t('language')} (${locale})`" right>
                    <NuxtLink v-for="locale in locales" :key="locale.code" :to="switchLocalePath(locale.code)"
                        class="dropdown-item icon-link">
                        <Icon name="ph:translate" aria-label="Language selection" />
                        {{ locale.name }}
                    </NuxtLink>
                </BNavItemDropdown>
            </BNavbarNav>
        </BNavbar>
        <BNavbar id="navbar" toggleable="lg" class="shadow-sm">
            <BNavbarBrand href="#" id="brand">
                <NuxtLink :to="localePath('/')">
                    <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" height="45" />
                    <span>Duty Duck</span>
                </NuxtLink>
            </BNavbarBrand>
            <BNavbarToggle target="nav-collapse" />
            <BCollapse id="nav-collapse" is-nav>
                <BNavbarNav class="mx-auto">
                    <NuxtLink class="nav-link" :to="localePath('/platform')">
                        Platform
                    </NuxtLink>
                    <NuxtLink class="nav-link" :to="localePath('/pricing')">
                        Pricing
                    </NuxtLink>
                    <NuxtLink class="nav-link" :to="localePath('/docs')">
                        Documentation
                    </NuxtLink>
                    <NuxtLink class="nav-link" :to="localePath('/blog')">
                        Blog
                    </NuxtLink>
                </BNavbarNav>
                <BNavbarNav>
                    <!-- Use a regular anchor tag here to avoid issue with pre-rendering the homepage and runtimeConfig -->
                    <a :href="localePath('/dashboard')" class="btn btn-outline-primary">
                        <span>Dashboard</span>
                    </a>
                </BNavbarNav>
            </BCollapse>
        </BNavbar>
    </div>


</template>

<style lang="scss" scoped>
@import "~/assets/main.scss";


#settings-bar {
    height: $settings-bar-height;
    background-color: $gray-200;
    font-size: 0.85rem;
}

#navbar {
    background-color: white;
    z-index: 1;
    padding: 0;

    @include media-breakpoint-down(sm) {
        :deep(.navbar-collapse.show) {
            padding-bottom: 1rem;
        }
    }
}


#brand {
    height: $navbar-height;
    overflow: hidden;
    margin: 0;
    position: relative;
    left: -10px;

    a {
        text-decoration: none;
    }

    &:hover {
        img {
            bottom: -3px;
        }
    }

    img {
        transition: bottom 100ms ease-in-out;
        height: $navbar-height - 5px;
        position: relative;
        bottom: -9px;
        margin-right: 16px;
    }
}
</style>