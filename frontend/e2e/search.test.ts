import { test, expect } from './fixtures';
import { overrideHandler } from './helpers';

test.describe('Search Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/search', { waitUntil: 'domcontentloaded' });
		await expect(page.getByRole('heading', { name: /rule search/i })).toBeVisible({
			timeout: 15_000
		});
	});

	test('initial prompt to select game', async ({ page }) => {
		await expect(page.getByText(/select a game/i).first()).toBeVisible();
	});

	test('search results with similarity scores', async ({ page }) => {
		// Select a game first
		await page.getByText('Catan').first().click();
		// Fill in search query and submit
		await page.getByPlaceholder(/win conditions|combat|movement/i).fill('victory points');
		// Use exact name to avoid matching the nav "Search Games" button
		await page.locator('form button[type="submit"]').click();

		// Should show results
		await expect(page.getByText(/victory points are earned/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByText('95%')).toBeVisible();
		await expect(page.getByText('2 matches')).toBeVisible();
	});

	test('no matches message', async ({ page }) => {
		// Select game first
		await page.getByText('Catan').first().click();

		// Override search to return empty results (correct endpoint)
		await overrideHandler(page, 'get', '/api/chat/search-rules', {
			body: {
				game_id: 1,
				query: 'xyz',
				results: [],
				total_results: 0
			}
		});

		await page.getByPlaceholder(/win conditions|combat|movement/i).fill('xyz');
		await page.locator('form button[type="submit"]').click();
		await expect(page.getByText(/no matches found/i)).toBeVisible({ timeout: 10_000 });
	});

	test('game selection works', async ({ page }) => {
		// Click on Catan in the game list
		await page.getByText('Catan').first().click();
		// Selected game info should appear
		await expect(page.getByText('Kosmos').first()).toBeVisible({ timeout: 5_000 });
	});
});
