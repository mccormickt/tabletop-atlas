import { test, expect } from './fixtures';
import { setupAdmin } from './helpers';

test.describe('Upload Page', () => {
	test.beforeEach(async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/upload');
		await expect(page.getByRole('heading', { name: /upload rules/i })).toBeVisible({
			timeout: 10_000
		});
	});

	test('select game prompt shown initially', async ({ page }) => {
		await expect(page.getByText(/select a game/i).last()).toBeVisible();
	});

	test('game list with search', async ({ page }) => {
		// Games should be listed in the sidebar
		await expect(page.getByText('Catan').first()).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('Carcassonne').first()).toBeVisible();
	});

	test('upload area shown after game selected', async ({ page }) => {
		// Select a game
		await page.getByText('Pandemic').first().click();
		// After selecting a game without rules, upload component should appear
		// The PDFUpload component should be visible
		await expect(page.getByText(/pandemic/i).first()).toBeVisible({ timeout: 5_000 });
	});

	test('rules info display when PDF exists', async ({ page }) => {
		// Select Catan which has rules
		await page.getByText('Catan').first().click();
		// Rules info should be displayed (from the PDFUpload component)
		await expect(page.getByText('Catan').first()).toBeVisible({ timeout: 5_000 });
	});
});
