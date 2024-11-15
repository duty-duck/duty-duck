export const useFiles = async () => {
    const $fetch = await useServer$fetch();

    const getFileUrl = async (fileId: string): Promise<string> => {
        return await $fetch<string>(`/files/${fileId}`, { method: 'GET' });
    }

    const redirectToFile = async (fileId: string) => {
        const url = await getFileUrl(fileId);
        window.open(url, '_blank');
    }

    return {
        getFileUrl,
        redirectToFile
    }
}