<template>
    <ShowcaseLayout>
        <div class="offcanvas offcanvas-start" tabindex="-1" id="docs-offcanvas">
            <div class="offcanvas-header">
                <button type="button" class="btn-close" data-bs-dismiss="offcanvas" aria-label="Close"></button>
            </div>
            <div class="offcanvas-body">
                <ShowcaseDocumentationSidebar />
            </div>
        </div>
        <div id="docs-container">
            <div id="docs-nav">
                <ShowcaseDocumentationSidebar />
            </div>
            <div id="docs-content">
                <slot></slot>
            </div>
        </div>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import '~/assets/main.scss';

#docs-container {
    display: flex;
    min-height: calc(100vh - $navbar-height);
}

#docs-nav {
    display: none;
    background-color: white;
    z-index: 0;
    border-right: 1px solid var(--bs-gray-200);
    overflow-y: auto;
    width: 280px;
    flex-shrink: 0;

    @include media-breakpoint-up(lg) {
        display: block;
    }
}

#docs-content {
    flex: 1;
}
</style>