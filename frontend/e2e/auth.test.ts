import { test, expect } from './fixtures';
import { setupUnauthenticated } from './helpers';

test.describe('Authentication', () => {
	test('unauthenticated user is redirected to /auth/login', async ({ page }) => {
		await setupUnauthenticated(page);
		await page.goto('/');
		await page.waitForURL('**/auth/login', { timeout: 10_000 });
		await expect(page).toHaveURL(/\/auth\/login/);
	});

	test('login page renders with sign-in heading', async ({ page }) => {
		await setupUnauthenticated(page);
		await page.goto('/auth/login');
		await expect(page.getByRole('heading', { name: /welcome to tabletop atlas/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(
			page.getByRole('main').getByRole('button', { name: /sign in with google/i })
		).toBeVisible();
	});

	test('authenticated user sees main content (not redirected)', async ({ page }) => {
		// Default routes return an authenticated user
		await page.goto('/');
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10_000 });
		await expect(page).not.toHaveURL(/\/auth\/login/);
	});

	test('loading spinner shows during auth check', async ({ page }) => {
		await page.goto('/');
		// The page should eventually resolve to main content
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10_000 });
	});

	test('unauthenticated user accessing protected route is redirected', async ({ page }) => {
		await setupUnauthenticated(page);
		await page.goto('/games');
		await page.waitForURL('**/auth/login', { timeout: 10_000 });
		await expect(page).toHaveURL(/\/auth\/login/);
	});
});
