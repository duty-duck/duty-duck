<script setup lang="ts">
const localePath = useLocalePath();
</script>

<template>
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
                <BButton :to="localePath('/dashboard')" variant="outline-primary">
                    <span>Dashboard</span>
                </BButton>
            </BNavbarNav>
        </BCollapse>

    </BNavbar>
</template>

<style lang="scss" scoped>
$navbar-height: 50px;

#navbar {
    background-color: white;
    padding: 0;
    z-index: 1;
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