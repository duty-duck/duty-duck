import { describe, test } from 'vitest'
import { setup } from './';
import { createPage } from '@nuxt/test-utils';
import { expect, type Page } from '@playwright/test';
import { rand } from '@vueuse/core';

export const fakeUser = {
    firstname: "Jane",
    lastName: "Doe",
    emailAddress: "jane@e2etests.com",
    password: "5pSQox3rLDxV@3AMx",
    orgName: "Tests Inc."
}

describe('Signup spec', async () => {
    await setup();

    test('user should be able to sign up', { timeout: 30000 }, async () => {
        await signUp({
            ...fakeUser,
            // use a random e-mail to avoid conflicts
            emailAddress: `test${rand(0, 10e9)}@e2etests.com`
        })
    });
})

async function signUp(user: typeof fakeUser) {
    // Fill the form
    const page = await createPage('/en/signup');
    await page.getByRole('textbox', { name: 'First name ' }).fill(user.firstname);
    await page.getByRole('textbox', { name: 'Last name ' }).fill(user.lastName);
    await page.getByRole('textbox', { name: 'E-mail' }).fill(user.emailAddress);
    await page.getByRole('textbox', { name: 'Password', exact: true }).fill(user.password);
    await page.getByRole('textbox', { name: 'Password confirmation', exact: true }).fill(user.password);
    await page.getByRole('textbox', { name: 'Your organization' }).fill(user.orgName);
    await page.getByRole('button', { name: 'Sign up' }).click();

    // Wait for comfirmation
    await expect(page.getByRole('heading', { name: 'Thank you for registering' })).toBeVisible();
    await page.getByRole('button', { name: 'Go to your dashboard' }).click();
    await expect(page.getByRole('heading', { name: 'Sign in to your account' })).toBeVisible();

    // Log in
    await page.getByRole('textbox', { name: 'email' }).fill(user.emailAddress);
    await page.getByRole('textbox', { name: 'Password' }).fill(user.password);
    await page.getByRole('button', { name: 'Sign in' }).click();

    // Wait for e-mail confirmation notice
    await expect(page.getByRole("heading", { name: "Email verification" })).toBeVisible();

    // Go to the mailbox
    await page.goto("http://localhost:1080");
    await page.getByRole('link', { name: 'Verify email' }).first().click();
    await page.frameLocator('.panel-html').getByRole('link', { name: 'Link to e-mail address verification' }).click();

    await new Promise(resolve => setTimeout(resolve, 2000));
    // Assert that the new user is signed in 
    await page.goto('http://localhost:5173/en/dashboard');
    await expect(page.locator('#auth-menu a.dropdown-toggle')).toBeVisible();
    await expect(page.locator('#auth-menu a.dropdown-toggle')).toHaveText(`${user.firstname} ${user.lastName}`);
}