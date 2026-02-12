import { test, expect } from './fixtures';
import { overrideHandler, setupUnauthenticated } from './helpers';

test.describe('Custom Games', () => {
	test('custom games tab renders list', async ({ page }) => {
		await page.goto('/games?tab=custom');
		await expect(page.getByRole('heading', { name: /game library/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('My Custom Card Game')).toBeVisible({ timeout: 5_000 });
	});

	test('empty state with create button', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/custom-games', {
			body: { items: [], page: 1, limit: 24, total: 0, total_pages: 0 }
		});
		await page.goto('/games?tab=custom');
		await expect(page.getByText(/no custom games yet/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /create your first/i })).toBeVisible();
	});

	test('create form at /games/custom/add', async ({ page }) => {
		await page.goto('/games/custom/add');
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10_000 });
	});

	test('sign in required for unauthenticated', async ({ page }) => {
		await setupUnauthenticated(page);
		await page.goto('/games?tab=custom');
		// Should redirect to login since entire app requires auth
		await page.waitForURL('**/auth/login', { timeout: 10_000 });
	});
});
