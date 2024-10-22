<script setup lang="ts">
const localePath = useLocalePath();
</script>
<template>
    <footer class="bg-dark py-5">
        <BContainer>
            <div class="row">
                <div class="col-md-4">

                </div>
                <div class="col-md-4">
                    <h5 class="text-secondary">Product</h5>
                </div>
                <div class="col-md-4">
                    <h5 class="text-secondary">Company</h5>
                    <BNav vertical>
                        <BNavItem :to="localePath('/')">Home</BNavItem>
                        <BNavItem :to="localePath('/blog')">Blog</BNavItem>
                        <BNavItem :to="localePath('/about-us')">About Us</BNavItem>
                    </BNav>

                </div>
            </div>
        </BContainer>
    </footer>
</template>

<style scoped lang="scss">
footer {
    .nav-link {
        padding-left: 0 !important;
        color: white;
    }
}
</style>
