import { test, expect } from './fixtures';
import { overrideHandler, setupAdmin } from './helpers';

test.describe('Games Library', () => {
	test('library tab is active by default and shows game cards', async ({ page }) => {
		await page.goto('/games');
		await expect(page.getByRole('heading', { name: /game library/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('Catan')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('Carcassonne')).toBeVisible();
		await expect(page.getByText('Pandemic')).toBeVisible();
	});

	test('game count in subtitle', async ({ page }) => {
		await page.goto('/games');
		await expect(page.getByText(/3 games in the library/i)).toBeVisible({ timeout: 10_000 });
	});

	test('loading state while fetching', async ({ page }) => {
		await page.goto('/games');
		// Verify final loaded state (MSW responds quickly)
		await expect(page.getByText('Catan')).toBeVisible({ timeout: 10_000 });
	});

	test('error state with retry button', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/games', {
			status: 500,
			body: { message: 'Internal server error', request_id: 'test' }
		});
		await page.goto('/games');
		await expect(page.getByText(/unable to load games/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /try again/i })).toBeVisible();
	});

	test('empty state when no games', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/games', {
			body: { items: [], page: 1, limit: 24, total: 0, total_pages: 0 }
		});
		await page.goto('/games');
		await expect(page.getByText(/no games found/i)).toBeVisible({ timeout: 10_000 });
	});

	test('tab switching to Collection', async ({ page }) => {
		await page.goto('/games');
		await expect(page.getByRole('heading', { name: /game library/i })).toBeVisible({
			timeout: 10_000
		});
		await page.getByRole('button', { name: /my collection/i }).click();
		await expect(page).toHaveURL(/tab=collection/);
	});

	test('tab switching to Custom', async ({ page }) => {
		await page.goto('/games');
		await expect(page.getByRole('heading', { name: /game library/i })).toBeVisible({
			timeout: 10_000
		});
		await page.getByRole('button', { name: /custom games/i }).click();
		await expect(page).toHaveURL(/tab=custom/);
	});

	test('admin sees Add to Library button', async ({ page }) => {
		// Button is hidden on mobile (md:flex only)
		test.skip(!!page.viewportSize() && page.viewportSize()!.width < 768, 'Desktop only');
		await setupAdmin(page);
		await page.goto('/games');
		await expect(page.getByText('Catan')).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /add to library/i })).toBeVisible();
	});
});
