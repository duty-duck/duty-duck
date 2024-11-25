import { setup as nuxtSetup } from '@nuxt/test-utils/e2e'
import type { TestOptions } from '@nuxt/test-utils';

export const config: Partial<TestOptions> & { mailbox: string, host: string } = {
    host: process.env.E2E_TESTS_HOST || 'https://preprod.dutyduck.net',
    mailbox: process.env.E2E_TESTS_MAILBOX || 'https://maildev.dutyduck.net',
    browser: true,
    browserOptions: {
        type: 'chromium',
        launch: {
            headless: process.env.E2E_TESTS_HEADLESS === 'false' ? false : true,
        }
    }
}

export const setup = async () => {
    return await nuxtSetup(config);
}

export const fakeUser = {
    firstname: "Jane",
    lastName: "Doe",
    emailAddress: "jane@e2etests.com",
    password: "5pSQox3rLDxV@3AMx",
    orgName: "Tests Inc."
}
