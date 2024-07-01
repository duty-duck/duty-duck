
<template>
    <div class="container d-flex align-items-center" style="min-height: 100vh;">
        <div class="row gx-lg-6">
            <div class="col-lg-6 col-xl-5 pe-xl-5">
                <img src="/assets/navbar-duck.png" alt="Duty Duck Logo" height="80px">
                <h1 class="mt-4 mb-3">Start monitoring your website today</h1>
                <p class="d-none d-md-block">Lorem ipsum dolor sit amet consectetur adipisicing elit. Rerum repudiandae laudantium cumque ex
                    velit, autem fuga quaerat! Voluptas voluptate ipsa exercitationem reprehenderit, aspernatur
                    perspiciatis non beatae quidem, maxime rem accusamus?</p>
                <p>
                    <NuxtLink to="/" class="icon-link">
                        <FeatherIcon type="arrow-left" />
                        Learn more
                    </NuxtLink>
                </p>
            </div>
            <div class="col-lg-6">
                <SignUpForm />
            </div>
        </div>
    </div>
</template>