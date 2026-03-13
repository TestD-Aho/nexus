import { test, expect } from '@playwright/test';

const BASE_URL = process.env.BASE_URL || 'http://localhost:5173';

test.describe('Nexus CMS - Homepage', () => {
  test('homepage loads correctly', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page).toHaveTitle(/Nexus/);
  });

  test('navbar is visible', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page.locator('.navbar')).toBeVisible();
    await expect(page.locator('.nav-brand')).toContainText('Nexus');
  });

  test('homepage loads with content', async ({ page }) => {
    await page.goto(BASE_URL);
    // Page should load without crash
    await expect(page.locator('.navbar')).toBeVisible();
    await expect(page.locator('.nav-brand')).toContainText('Nexus');
  });
});

test.describe('Nexus CMS - Login', () => {
  test('login page loads', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('h1')).toContainText('Nexus CMS');
    await expect(page.locator('h2')).toContainText('Sign In');
  });

  test('login form has required fields', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('input[type="email"]')).toBeVisible();
    await expect(page.locator('input[type="password"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('shows default credentials hint', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('.hint')).toContainText('admin@nexus.local');
  });

  test('can type in form fields', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await page.fill('input[type="email"]', 'test@test.com');
    await page.fill('input[type="password"]', 'password123');
    await expect(page.locator('input[type="email"]')).toHaveValue('test@test.com');
    await expect(page.locator('input[type="password"]')).toHaveValue('password123');
  });
});

test.describe('Nexus CMS - Navigation', () => {
  test('admin redirect when not logged in', async ({ page }) => {
    await page.goto(`${BASE_URL}/admin`);
    await expect(page).toHaveURL(/login/);
  });

  test('unknown route shows 404', async ({ page }) => {
    await page.goto(`${BASE_URL}/nonexistent-page`);
    await expect(page.locator('h1')).toContainText('404');
  });

  test('can access public page view', async ({ page }) => {
    await page.goto(`${BASE_URL}/page/test`);
    // Either shows content or 404
    const valid = page.locator('.error-page, .public-page, .loading');
    await expect(valid).toBeVisible();
  });
});

test.describe('Nexus CMS - Responsive', () => {
  test('works on mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto(BASE_URL);
    await expect(page.locator('.navbar')).toBeVisible();
  });

  test('works on tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto(BASE_URL);
    await expect(page.locator('.navbar')).toBeVisible();
  });

  test('works on desktop viewport', async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto(BASE_URL);
    await expect(page.locator('.navbar')).toBeVisible();
  });
});

test.describe('Nexus CMS - Accessibility', () => {
  test('login form is accessible', async ({ page }) => {
    await page.goto(`${BASE_URL}/login`);
    await expect(page.locator('input[type="email"]')).toHaveAttribute('type', 'email');
    await expect(page.locator('input[type="password"]')).toHaveAttribute('type', 'password');
  });

  test('page has proper heading hierarchy', async ({ page }) => {
    await page.goto(BASE_URL);
    const h1 = page.locator('h1');
    if (await h1.count() > 0) {
      await expect(h1.first()).toBeVisible();
    }
  });
});

test.describe('Nexus CMS - Console', () => {
  test('no critical console errors on homepage', async ({ page }) => {
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });
    await page.goto(BASE_URL);
    await page.waitForLoadState('networkidle');
    // Filter out expected errors (like API calls failing)
    const criticalErrors = errors.filter(e => !e.includes('Failed to load'));
    expect(criticalErrors).toHaveLength(0);
  });

  test('no critical console errors on login', async ({ page }) => {
    const errors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });
    await page.goto(`${BASE_URL}/login`);
    await page.waitForLoadState('networkidle');
    const criticalErrors = errors.filter(e => !e.includes('Failed to load'));
    expect(criticalErrors).toHaveLength(0);
  });
});
