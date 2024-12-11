
export const useDocumentationFirstPage = async () => {
    const { locale } = useI18n()
    const localePath = useLocalePath()
    const { data } = await useAsyncData('docs-home', () => queryContent('docs', locale.value).find())

    return computed(() => {
        const path = data.value![0]!._path!.replace(`/${locale.value}/`, '/')
        return localePath(path)
    });
};

export const useCurrentContentPath = () => {
    const { locale } = useI18n()
    const route = useRoute()

    return computed(() => {
        let section = 'docs';
        if (route.path.includes('/blog/')) {
            section = 'blog';
        }
        const slug: string[] = route.params.slug as string[];
        if (!slug || slug.length == 0) {
            return ""
        }
        return ["", section, locale.value, ...slug].join('/')
    })
}

export type NavigationLink = {
    _path: string;
    title: string;
}

export const useNextAndPrevious = async () => {
    const currentPath = useCurrentContentPath();
    const { locale } = useI18n();

    return await useAsyncData('navigation', async () => {
        const docs = await queryContent('docs', locale.value).only(['_path', 'title']).find();
        const currentIndex = docs.findIndex(doc => doc._path === currentPath.value);
        return {
            prev: currentIndex > 0 ? docs[currentIndex - 1] as NavigationLink : null,
            next: currentIndex < docs.length - 1 ? docs[currentIndex + 1] as NavigationLink : null
        }
    })
}

/**
 * Compute the href attribute a Nuxt content link should have (used for blog posts and docs)
 */
export const useComputeContentLinkDest = () => {
    const { locale } = useI18n();
    const localePath = useLocalePath();

    return (item: { _path: string }) => {
        return localePath(item._path.replace(`/${locale.value}/`, '/'))
    }
}