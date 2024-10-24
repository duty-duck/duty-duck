<script setup lang="ts">
const localePath = useLocalePath();
</script>
<template>
    <footer class="bg-dark py-5">
        <BContainer>
            <div class="row row-gap-5">
                <div class="footer-col">
                    <div id="footer-brand">
                        <div class="img-container">
                            <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" height="50" />
                        </div>
                        <h5>Duty Duck</h5>
                    </div>
                </div>
                <div class="footer-col">
                    <h5 class="text-secondary">Product</h5>
                    <NuxtLink>On-call management</NuxtLink>
                    <NuxtLink>Alerting</NuxtLink>
                    <NuxtLink>Incident management</NuxtLink>
                    <NuxtLink>Status pages</NuxtLink>
                    <NuxtLink>Pricing</NuxtLink>
                </div>
                <div class="footer-col">
                    <h5 class="text-secondary">Company</h5>
                    <NuxtLink>Home</NuxtLink>
                    <NuxtLink>Blog</NuxtLink>
                    <NuxtLink>About Us</NuxtLink>
                </div>
            </div>
        </BContainer>
    </footer>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";

$img-height: 50px;

footer {
    #footer-brand {
        color: white;
        width: 100%;

        .img-container {
            position: relative;
            border-bottom: 1px solid $gray-600;
            max-width: 220px;
            overflow: hidden;
            margin-bottom: .3rem;

            img {
                position: relative;
                left: -15px;
                bottom: -4px;
                height: $img-height;
            }
        }
    }

    @include media-breakpoint-up(md) {
        #footer-brand {
            position: absolute;
            transition: top 100ms ease-in-out;
            top: -$img-height;

            &:hover {
                top: 0;
            }
        }

    }

    .footer-col {
        @extend .col-md-4;
        display: flex;
        flex-direction: column;
        gap: .25rem;
        position: relative;
        overflow: hidden;

        a {
            color: $primary !important;
            text-decoration: none;
        }
    }
}
</style>
