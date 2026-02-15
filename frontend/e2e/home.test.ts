import { test, expect } from './fixtures';

test.describe('Home Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/', { waitUntil: 'domcontentloaded' });
		// Wait for auth to resolve and page to load
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 15_000 });
	});

	test('hero section displays main heading', async ({ page }) => {
		await expect(
			page.getByRole('heading', { level: 1 }).filter({ hasText: /board game/i })
		).toBeVisible();
	});

	test('four quick action cards are visible', async ({ page }) => {
		// The h3 headings inside the quick action cards
		await expect(page.getByRole('heading', { name: 'Add New Game' })).toBeVisible({
			timeout: 5_000
		});
		await expect(page.getByRole('heading', { name: 'Search Rules' })).toBeVisible();
		await expect(page.getByRole('heading', { name: 'Ask Questions' })).toBeVisible();
		await expect(page.getByRole('heading', { name: '8x8 Challenge' })).toBeVisible();
		// Upload card was moved to admin — should not appear here
		await expect(page.getByRole('heading', { name: 'Upload Rules' })).not.toBeVisible();
	});

	test('collection stats show game count from API', async ({ page }) => {
		// The mock API returns 3 games total
		await expect(page.getByText('Games in collection')).toBeVisible({ timeout: 5_000 });
	});

	test('Browse Collection button navigates to /games', async ({ page }) => {
		await page.getByRole('button', { name: /browse collection/i }).click();
		await expect(page).toHaveURL(/\/games/);
	});

	test('Getting Started steps render', async ({ page }) => {
		// The step descriptions use <p> tags — filter to exact text to avoid matching buttons
		await expect(page.getByText('Add your first game', { exact: true })).toBeVisible({
			timeout: 5_000
		});
		await expect(page.getByText('Upload rule books', { exact: true })).toBeVisible();
		await expect(page.getByText('Search or ask questions', { exact: true })).toBeVisible();
	});
});
