import { test as base, type Page } from '@playwright/test';
import * as data from '../src/mocks/data';

/** Shape of a per-test route override stored on the page object. */
export type RouteOverride = {
	method: string;
	pathPattern: string;
	status: number;
	body: unknown;
};

/** Key used to store override array on the Playwright Page object. */
const OVERRIDES_KEY = '__routeOverrides';

/** Retrieve the mutable override list from a page (used by helpers). */
export function getOverrides(page: Page): RouteOverride[] {
	return (page as Record<string, unknown>)[OVERRIDES_KEY] as RouteOverride[];
}

function json(body: unknown, status = 200) {
	return {
		status,
		contentType: 'application/json',
		body: status === 204 ? '' : JSON.stringify(body)
	};
}

/**
 * Check if a request path matches an override path pattern.
 * Supports exact paths like "/api/games" and patterns with :param like "/api/games/:id".
 */
function pathMatches(pattern: string, actual: string): boolean {
	// Convert :param patterns to regex
	const regexStr = '^' + pattern.replace(/:(\w+)/g, '[^/]+') + '$';
	return new RegExp(regexStr).test(actual);
}

/**
 * Single catch-all route handler for all /api/** requests.
 * Checks per-test overrides first (last added wins), then falls through to defaults.
 */
async function setupDefaultRoutes(page: Page) {
	// Initialize the override list on the page object
	(page as Record<string, unknown>)[OVERRIDES_KEY] = [] as RouteOverride[];

	await page.route('**/api/**', async (route) => {
		const url = new URL(route.request().url());
		const path = url.pathname;
		const method = route.request().method().toUpperCase();

		// --- Check per-test overrides first (last added = highest priority) ---
		const overrides = getOverrides(page);
		for (let i = overrides.length - 1; i >= 0; i--) {
			const o = overrides[i];
			if (pathMatches(o.pathPattern, path) && method === o.method.toUpperCase()) {
				return route.fulfill(json(o.body, o.status));
			}
		}

		// --- Default handlers (authenticated happy path) ---

		// Auth
		if (path === '/api/auth/me' && method === 'GET')
			return route.fulfill(json({ user: data.mockUser }));
		if (path === '/api/auth/refresh' && method === 'POST') return route.fulfill(json({}));
		if (path === '/api/auth/logout' && method === 'POST') return route.fulfill(json({}));

		// Games list/create
		if (path === '/api/games' && method === 'GET') {
			const search = url.searchParams.get('search');
			return route.fulfill(json(search ? data.mockFilteredGamesPage(search) : data.mockGamesPage));
		}
		if (path === '/api/games' && method === 'POST')
			return route.fulfill(json(data.mockGameDetail, 201));

		// Game detail
		if (/^\/api\/games\/\d+$/.test(path)) {
			if (method === 'GET') return route.fulfill(json(data.mockGameDetail));
			if (method === 'PUT') return route.fulfill(json(data.mockGameDetail));
			if (method === 'DELETE') return route.fulfill(json(null, 204));
		}

		// Rules info
		if (/^\/api\/games\/\d+\/rules-info$/.test(path) && method === 'GET') {
			const id = parseInt(path.split('/')[3]);
			return route.fulfill(json(id === 1 ? data.mockRulesInfo : data.mockNoRulesInfo));
		}
		if (/^\/api\/games\/\d+\/rules-upload$/.test(path) && method === 'POST')
			return route.fulfill(
				json({
					message: 'Rules uploaded',
					file_path: '/rules/test.pdf',
					text_length: 5000,
					chunks_processed: 20
				})
			);
		if (/^\/api\/games\/\d+\/rules$/.test(path) && method === 'DELETE')
			return route.fulfill(
				json({ message: 'Rules deleted', file_deleted: true, embeddings_deleted: 20 })
			);

		// House Rules
		if (path === '/api/house-rules' && method === 'GET')
			return route.fulfill(json(data.mockHouseRulesPage));
		if (path === '/api/house-rules' && method === 'POST')
			return route.fulfill(json(data.mockHouseRulesPage.items[0], 201));
		if (/^\/api\/house-rules\/\d+$/.test(path)) {
			if (method === 'PUT') return route.fulfill(json(data.mockHouseRulesPage.items[0]));
			if (method === 'DELETE') return route.fulfill(json(null, 204));
		}

		// Chat
		if (path === '/api/chat/sessions' && method === 'GET')
			return route.fulfill(json(data.mockChatSessions));
		if (path === '/api/chat/sessions' && method === 'POST')
			return route.fulfill(json(data.mockNewSession, 201));
		if (/^\/api\/chat\/sessions\/\d+$/.test(path)) {
			if (method === 'GET') return route.fulfill(json(data.mockChatHistory));
			if (method === 'PUT') return route.fulfill(json(data.mockChatHistory.session));
		}
		if (path === '/api/chat' && method === 'POST')
			return route.fulfill(json(data.mockChatResponse));

		// Search
		if (path === '/api/chat/search-rules' && method === 'GET')
			return route.fulfill(json(data.mockSearchResults));

		// Collection
		if (path === '/api/collection' && method === 'GET')
			return route.fulfill(json(data.mockCollectionPage));
		if (path === '/api/collection' && method === 'POST')
			return route.fulfill(json(data.mockCollectionPage.items[0], 201));
		if (/^\/api\/collection\/\d+$/.test(path) && method === 'DELETE')
			return route.fulfill(json(null, 204));

		// Custom Games
		if (path === '/api/custom-games' && method === 'GET')
			return route.fulfill(json(data.mockCustomGamesPage));
		if (path === '/api/custom-games' && method === 'POST')
			return route.fulfill(json(data.mockCustomGamesPage.items[0], 201));
		if (/^\/api\/custom-games\/\d+$/.test(path) && method === 'DELETE')
			return route.fulfill(json(null, 204));

		// Challenges
		if (path === '/api/challenges' && method === 'GET')
			return route.fulfill(json(data.mockChallengesPage));
		if (path === '/api/challenges' && method === 'POST')
			return route.fulfill(json(data.mockChallengeDetail, 201));
		const challengeMatch = path.match(/^\/api\/challenges\/(\d+)(\/.*)?$/);
		if (challengeMatch) {
			const sub = challengeMatch[2];
			if (!sub && method === 'GET') return route.fulfill(json(data.mockChallengeDetail));
			if (sub === '/grid' && method === 'GET')
				return route.fulfill(json(data.mockChallengeGridView));
			if (sub === '/stats' && method === 'GET') return route.fulfill(json(data.mockChallengeStats));
			if (sub === '/participants' && method === 'POST') return route.fulfill(json({}, 201));
			if (sub === '/games' && method === 'POST') return route.fulfill(json({}, 201));
			if (sub === '/plays' && method === 'POST') return route.fulfill(json({}, 201));
		}

		// Tools
		if (path === '/api/tools' && method === 'GET') return route.fulfill(json(data.mockToolsList));
		const toolMatch = path.match(/^\/api\/tools\/([^/]+)(\/.*)?$/);
		if (toolMatch) {
			const sub = toolMatch[2];
			if (!sub && method === 'GET') return route.fulfill(json(data.mockToolDetail));
			if (sub === '/calculate' && method === 'POST')
				return route.fulfill(json(data.mockScoreOutput));
		}

		// Admin
		if (path === '/api/admin/stats' && method === 'GET')
			return route.fulfill(json(data.mockAdminStats));
		if (path === '/api/admin/bgg/stats' && method === 'GET')
			return route.fulfill(json(data.mockEnrichmentStats));

		// Unhandled API request — let through
		return route.fallback();
	});
}

// Fixture that sets up API mocking for every test
export const test = base.extend<{ mockApi: void }>({
	mockApi: [
		async ({ page }, use) => {
			await setupDefaultRoutes(page);
			await use();
		},
		{ auto: true }
	]
});

export { expect, type Page } from '@playwright/test';
