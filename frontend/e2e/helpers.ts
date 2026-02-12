import type { Page } from '@playwright/test';
import { getOverrides } from './fixtures';

/**
 * Override an API endpoint for the current test.
 * Pushes to the shared override list checked by the catch-all handler in fixtures.ts.
 * Later overrides take priority (last added wins).
 * Can be called before or after page.goto().
 */
export function overrideHandler(
	page: Page,
	method: 'get' | 'post' | 'put' | 'patch' | 'delete',
	path: string,
	response: { status?: number; body: unknown }
) {
	getOverrides(page).push({
		method,
		pathPattern: path,
		status: response.status ?? 200,
		body: response.body
	});
}

/** Setup unauthenticated state — call before page.goto() */
export function setupUnauthenticated(page: Page) {
	overrideHandler(page, 'get', '/api/auth/me', {
		status: 401,
		body: { message: 'Unauthorized', request_id: 'test' }
	});
	overrideHandler(page, 'post', '/api/auth/refresh', {
		status: 401,
		body: { message: 'Unauthorized', request_id: 'test' }
	});
}

/** Setup admin auth — call before page.goto() */
export function setupAdmin(page: Page) {
	const adminUser = {
		id: 2,
		email: 'admin@test.com',
		display_name: 'Admin User',
		picture_url: null,
		role: 'admin'
	};
	overrideHandler(page, 'get', '/api/auth/me', {
		body: { user: adminUser }
	});
}
