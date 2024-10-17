<script setup lang="ts">
const localePath = useLocalePath();
</script>

<template>
    <BNavbar id="navbar" toggleable="lg">
        <BNavbarBrand href="#" id="brand">
            <NuxtLink :to="localePath('/')">
                <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" />
                <span>Duty Duck</span>
            </NuxtLink>
        </BNavbarBrand>
        <BNavbarToggle target="nav-collapse" />
        <BCollapse id="nav-collapse" is-nav>
            <BNavbarNav class="mx-auto">
                <BNavItem :to="localePath('/platform')">
                    Platform
                </BNavItem>
                <BNavItem :to="localePath('/pricing')">
                    Pricing
                </BNavItem>
                <BNavItem :to="localePath('/docs')">
                    Documentation
                </BNavItem>
                <BNavItem :to="localePath('/blog')">
                    Blog
                </BNavItem>
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