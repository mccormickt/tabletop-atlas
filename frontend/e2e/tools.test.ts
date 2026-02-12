import { test, expect } from './fixtures';
import { overrideHandler } from './helpers';

test.describe('Tools Page', () => {
	test('tools catalog with cards', async ({ page }) => {
		await page.goto('/tools');
		await expect(page.getByRole('heading', { name: /game tools/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('7 Wonders Score Calculator')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('Carcassonne Score Calculator')).toBeVisible();
	});

	test('empty state message', async ({ page }) => {
		// Override tools to return empty before navigation
		await overrideHandler(page, 'get', '/api/tools', {
			body: []
		});
		await page.goto('/tools');
		await expect(page.getByRole('heading', { name: /game tools/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText(/no tools available/i)).toBeVisible({ timeout: 5_000 });
	});

	test('tool detail page renders', async ({ page }) => {
		await page.goto('/tools/7wonders', { waitUntil: 'domcontentloaded' });
		await expect(page.getByText('7 Wonders Score Calculator')).toBeVisible({ timeout: 15_000 });
		await expect(page.getByText('3-7 players', { exact: true })).toBeVisible();
	});

	test('tool detail shows game setup phase', async ({ page }) => {
		await page.goto('/tools/7wonders', { waitUntil: 'domcontentloaded' });
		await expect(page.getByText('7 Wonders Score Calculator')).toBeVisible({ timeout: 15_000 });
		// The ScoreCalculator starts in Setup phase with player names
		await expect(page.getByText('Game Setup')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('Number of Players')).toBeVisible();
	});
});
