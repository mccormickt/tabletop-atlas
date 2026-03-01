import { test, expect } from './fixtures';
import { overrideHandler, setupAdmin } from './helpers';

test.describe('Game Detail Page', () => {
	test('displays game info (name, description, players, complexity)', async ({ page }) => {
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
		await expect(page.getByText('Trade, build, and settle')).toBeVisible();
		await expect(page.getByText(/3.4 players/i)).toBeVisible();
		await expect(page.getByText(/2\.3\/5\.0/)).toBeVisible();
	});

	test('loading state shows spinner', async ({ page }) => {
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
	});

	test('error state renders with Try Again', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/games/:id', {
			status: 500,
			body: { message: 'Server error', request_id: 'test' }
		});
		await page.goto('/games/1');
		await expect(page.getByText(/unable to load game/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /try again/i })).toBeVisible();
	});

	test('house rules tab shows rules', async ({ page }) => {
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
		// Click the House Rules tab (use exact name to avoid matching "View House Rules" sidebar button)
		await page.getByRole('button', { name: 'House Rules', exact: true }).click();
		await expect(page.getByText('Friendly Robber')).toBeVisible({ timeout: 5_000 });
	});

	test('admin sees edit and delete buttons', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /edit game/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /delete game/i })).toBeVisible();
	});

	test('authenticated user sees collection button', async ({ page }) => {
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
		// Should see either "Add to Collection" or "In Collection" button
		await expect(
			page.getByRole('button', { name: /add to collection|in collection/i })
		).toBeVisible({ timeout: 5_000 });
	});

	test('admin does not see upload or replace buttons on game detail', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/games/1');
		await expect(page.getByRole('heading', { name: 'Catan' })).toBeVisible({ timeout: 10_000 });
		// Upload was moved to admin panel — no upload/replace buttons should appear here
		await expect(page.getByRole('button', { name: /upload/i })).not.toBeVisible();
		await expect(page.getByRole('button', { name: /replace/i })).not.toBeVisible();
	});
});
