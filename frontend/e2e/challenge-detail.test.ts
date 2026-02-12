import { test, expect } from './fixtures';

test.describe('Challenge Detail Page', () => {
	test('grid view renders with challenge name', async ({ page }) => {
		await page.goto('/challenges/1');
		await expect(page.getByRole('heading', { name: /board game night 2024/i })).toBeVisible({
			timeout: 10_000
		});
	});

	test('participants visible', async ({ page }) => {
		await page.goto('/challenges/1');
		await expect(page.getByRole('heading', { name: /board game night 2024/i })).toBeVisible({
			timeout: 10_000
		});
		// Participants sidebar
		await expect(page.getByText('Test User').first()).toBeVisible();
		await expect(page.getByText('Player Two').first()).toBeVisible();
	});

	test('stats page with leaderboard', async ({ page }) => {
		await page.goto('/challenges/1/stats');
		await expect(page.getByText(/stats/i).first()).toBeVisible({ timeout: 10_000 });
		// Completed and total cells
		await expect(page.getByText('Sessions Completed')).toBeVisible();
		await expect(page.getByText('Total Sessions')).toBeVisible();
	});

	test('loading state resolves', async ({ page }) => {
		await page.goto('/challenges/1');
		await expect(page.getByRole('heading', { name: /board game night 2024/i })).toBeVisible({
			timeout: 10_000
		});
	});
});
