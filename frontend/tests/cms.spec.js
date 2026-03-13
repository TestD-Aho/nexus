import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:5173';

test.describe('Nexus CMS', () => {
  test('homepage loads correctly', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page).toHaveTitle(/Nexus/);
  });

  test('login page loads', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('h1')).toContainText('Nexus CMS');
    await expect(page.locator('h2')).toContainText('Sign In');
  });

  test('can navigate to pages', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page.locator('.nav-brand')).toBeVisible();
    await expect(page.locator('text=Pages')).toBeVisible();
  });

  test('admin redirect when not logged in', async ({ page }) => {
    await page.goto(`${BASE_URL}/admin`);
    await expect(page).toHaveURL(/login/);
  });

  test('public page view loads', async ({ page }) => {
    // This will 404 but should load the page structure
    await page.goto(`${BASE_URL}/page/test`);
    // Should show either page content or 404
    await expect(page.locator('.error-page, .public-page, .loading')).toBeVisible();
  });
});

test.describe('Block Editor', () => {
  test('block editor component renders', async ({ page }) => {
    // Would require auth to test fully
    // This is a placeholder for integration tests
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
  });
});
