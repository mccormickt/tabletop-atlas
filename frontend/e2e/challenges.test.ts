import { test, expect } from './fixtures';
import { overrideHandler } from './helpers';

test.describe('Challenges Page', () => {
	test('challenge cards display', async ({ page }) => {
		await page.goto('/challenges');
		await expect(page.getByRole('heading', { name: /challenges/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('Board Game Night 2024')).toBeVisible({ timeout: 5_000 });
	});

	test('empty state with create action', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/challenges', {
			body: { items: [], page: 1, limit: 24, total: 0, total_pages: 0 }
		});
		await page.goto('/challenges');
		await expect(page.getByText(/no challenges yet/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /create challenge/i })).toBeVisible();
	});

	test('error state with retry', async ({ page }) => {
		await overrideHandler(page, 'get', '/api/challenges', {
			status: 500,
			body: { message: 'Server error' }
		});
		await page.goto('/challenges');
		await expect(page.getByText(/failed to load challenges/i)).toBeVisible({ timeout: 10_000 });
		await expect(page.getByRole('button', { name: /retry/i })).toBeVisible();
	});

	test('create challenge link visible', async ({ page }) => {
		await page.goto('/challenges');
		await expect(page.getByRole('heading', { name: /challenges/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByRole('link', { name: /create challenge/i })).toBeVisible();
	});
});
