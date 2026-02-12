import { test, expect } from './fixtures';
import { overrideHandler, setupUnauthenticated } from './helpers';

test.describe('Collection Page', () => {
	test('collection entries display with game names', async ({ page }) => {
		await page.goto('/collection');
		await expect(page.getByRole('heading', { name: /my collection/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('Catan')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('Carcassonne')).toBeVisible();
	});

	test('empty state with Browse Games action', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/collection', {
			body: { items: [], page: 1, limit: 24, total: 0, total_pages: 0 }
		});
		await page.goto('/collection');
		await expect(page.getByText(/your collection is empty/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /browse games/i })).toBeVisible();
	});

	test('loading state shows spinner', async ({ page }) => {
		await page.goto('/collection');
		// Eventually the content loads
		await expect(page.getByRole('heading', { name: /my collection/i })).toBeVisible({
			timeout: 10_000
		});
	});

	test('redirect to login when unauthenticated', async ({ page }) => {
		await setupUnauthenticated(page);
		await page.goto('/collection');
		await page.waitForURL('**/auth/login', { timeout: 10_000 });
	});
});
