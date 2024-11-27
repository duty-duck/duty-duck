
export const useDocumentationFirstPage = async () => {
    const { locale } = useI18n()
    const localePath = useLocalePath()
    const { data } = await useAsyncData('docs-home', () => queryContent('docs', locale.value).find())

    return computed(() => {
        const path = data.value![0]!._path!.replace(`/${locale.value}/`, '/')
        return localePath(path)
    });
};

export const useCurrentPath = () => {
    const { locale } = useI18n()
    const route = useRoute()
    return computed(() => {
        const slug: string[] = route.params.slug as string[];
        if (!slug || slug.length == 0) {
            return ""
        }
        return ["", "docs", locale.value, ...slug].join('/')
    })
}