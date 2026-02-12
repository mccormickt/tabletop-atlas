import { test, expect } from './fixtures';
import { setupAdmin } from './helpers';

test.describe('Admin Dashboard', () => {
	test('dashboard with Master Games count (admin auth)', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin');
		await expect(page.getByRole('heading', { name: /admin dashboard/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('Master Games')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('150')).toBeVisible();
	});

	test('non-admin user is redirected away from admin', async ({ page }) => {
		// Default user has role 'user' — admin layout redirects to home
		await page.goto('/admin');
		await page.waitForURL('**/', { timeout: 10_000 });
		await expect(page).not.toHaveURL(/\/admin/);
	});

	test('enrichment stats display', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin');
		await expect(page.getByRole('heading', { name: /admin dashboard/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('BGG Enrichment')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText(/need enrichment/)).toBeVisible();
	});

	test('import page renders', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/games/import');
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10_000 });
	});

	test('enrich page renders', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/games/enrich');
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible({ timeout: 10_000 });
	});
});
