import { test, expect } from './fixtures';
import { overrideHandler, setupAdmin, setupUnauthenticated } from './helpers';

test.describe('Admin User Management', () => {
	test('page renders with user list', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('3 users total')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('user@example.com')).toBeVisible();
		await expect(page.getByText('admin@test.com')).toBeVisible();
		await expect(page.getByText('admin2@test.com')).toBeVisible();
	});

	test('non-admin user is redirected away', async ({ page }) => {
		await page.goto('/admin/users');
		await page.waitForURL('**/', { timeout: 10_000 });
		await expect(page).not.toHaveURL(/\/admin/);
	});

	test('unauthenticated user is redirected to login', async ({ page }) => {
		setupUnauthenticated(page);
		await page.goto('/admin/users');
		await page.waitForURL('**/auth/login**', { timeout: 10_000 });
		await expect(page).toHaveURL(/\/auth\/login/);
	});

	test('search input triggers filtered API call', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/users');
		await expect(page.getByText('3 users total')).toBeVisible({ timeout: 10_000 });

		// Override to return filtered results before typing
		overrideHandler(page, 'get', '/api/admin/users', {
			body: {
				items: [
					{
						id: 2,
						email: 'admin@test.com',
						display_name: 'Admin User',
						role: 'admin',
						created_at: '2024-01-02T00:00:00Z'
					}
				],
				page: 1,
				limit: 20,
				total: 1,
				total_pages: 1
			}
		});

		// Type in the search input — triggers debounced API call
		await page.getByPlaceholder('Search by email or name...').fill('admin');
		await expect(page.getByText('1 user total')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText('admin@test.com')).toBeVisible();
		await expect(page.getByText('user@example.com')).not.toBeVisible();
	});

	test('own user row shows (You) badge and no dropdown', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		// Admin user (id: 2) matches the setupAdmin mock user
		await expect(page.getByText('(You)')).toBeVisible({ timeout: 5_000 });
		// The own-user row should NOT have a role select dropdown
		const ownRow = page.getByRole('row').filter({ hasText: 'admin@test.com' });
		await expect(ownRow.locator('select')).toHaveCount(0);
	});

	test('role change shows success message', async ({ page }) => {
		await setupAdmin(page);
		overrideHandler(page, 'put', '/api/admin/users/:id/role', {
			body: {
				id: 1,
				email: 'user@example.com',
				display_name: 'Regular User',
				role: 'admin',
				created_at: '2024-01-01T00:00:00Z'
			}
		});
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		// Find the role select for the first non-admin user (user@example.com)
		const firstUserRow = page.getByRole('row').filter({ hasText: 'user@example.com' });
		const roleSelect = firstUserRow.locator('select');
		await roleSelect.selectOption('admin');
		await expect(page.getByText('Role updated successfully')).toBeVisible({ timeout: 5_000 });
	});

	test('role change error reverts optimistic update', async ({ page }) => {
		await setupAdmin(page);
		overrideHandler(page, 'put', '/api/admin/users/:id/role', {
			status: 400,
			body: {
				message: 'Cannot demote the last remaining admin',
				request_id: 'test'
			}
		});
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		// Change an admin's role to user
		const adminRow = page.getByRole('row').filter({ hasText: 'admin2@test.com' });
		const roleSelect = adminRow.locator('select');
		await roleSelect.selectOption('user');
		// Error message should appear
		await expect(page.getByText(/Cannot demote/)).toBeVisible({ timeout: 5_000 });
		// Select should revert to original value
		await expect(roleSelect).toHaveValue('admin');
	});

	test('error state on initial load failure', async ({ page }) => {
		await setupAdmin(page);
		overrideHandler(page, 'get', '/api/admin/users', {
			status: 500,
			body: { message: 'Internal server error', request_id: 'test' }
		});
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText(/Failed to load users|Internal server error/)).toBeVisible({
			timeout: 5_000
		});
	});

	test('empty state when no users match', async ({ page }) => {
		await setupAdmin(page);
		overrideHandler(page, 'get', '/api/admin/users', {
			body: { items: [], page: 1, limit: 20, total: 0, total_pages: 0 }
		});
		await page.goto('/admin/users');
		await expect(page.getByRole('heading', { name: /user management/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('No users found.')).toBeVisible({ timeout: 5_000 });
	});

	test('dashboard has Manage Users link', async ({ page }) => {
		await setupAdmin(page);
		await page.goto('/admin');
		await expect(page.getByRole('heading', { name: /admin dashboard/i })).toBeVisible({
			timeout: 10_000
		});
		await expect(page.getByText('User Management')).toBeVisible({ timeout: 5_000 });
		const link = page.getByRole('link', { name: 'Manage Users' });
		await expect(link).toBeVisible();
		await expect(link).toHaveAttribute('href', /\/admin\/users/);
	});
});
