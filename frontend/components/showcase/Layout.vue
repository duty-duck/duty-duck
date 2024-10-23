<template>
    <div id="showcase-container">
        <ShowcaseNavbar />
        <div id="showcase-content">
            <slot />
        </div>
        <LazyShowcaseFooter />
    </div>
</template>

<style scoped lang="scss">
#showcase-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;

    #showcase-content {
        flex: 1;
    }
}
</style>