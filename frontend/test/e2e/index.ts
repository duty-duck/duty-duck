import { setup as nuxtSetup } from '@nuxt/test-utils/e2e'

export const setup = async () => {
    return await nuxtSetup({
        host: 'localhost:5173',
    });
}