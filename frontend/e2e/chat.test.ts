import { test, expect, type Page } from './fixtures';

function isMobile(page: Page) {
	const viewport = page.viewportSize();
	return viewport && viewport.width < 1024;
}

/** On mobile, the game list is in a drawer. Opens it and returns a scoped locator. */
async function ensureGameListVisible(page: Page) {
	if (isMobile(page)) {
		await page.getByRole('button', { name: /select game/i }).click();
		const drawer = page.locator('.fixed').filter({ has: page.getByText('Select Game') });
		await expect(drawer).toBeVisible({ timeout: 3_000 });
		return drawer;
	}
	return page;
}

/** On mobile, sessions are in a drawer accessed via "Sessions" button. */
async function ensureSessionsVisible(page: Page) {
	if (isMobile(page)) {
		await page.getByRole('button', { name: /sessions/i }).click();
		const drawer = page.locator('.fixed').filter({ has: page.getByText('Chat Sessions') });
		await expect(drawer).toBeVisible({ timeout: 3_000 });
		return drawer;
	}
	return page;
}

test.describe('Chat Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/chat');
		await expect(page.getByRole('heading', { name: /game rules chat/i })).toBeVisible({
			timeout: 10_000
		});
	});

	test('game selection area on initial load', async ({ page }) => {
		await expect(page.getByText(/select a game/i)).toBeVisible();
	});

	test('games list for selection from API', async ({ page }) => {
		const gameList = await ensureGameListVisible(page);
		await expect(gameList.getByText('Catan').first()).toBeVisible({ timeout: 5_000 });
	});

	test('chat messages display after selecting game and session', async ({ page }) => {
		// Select game
		const gameList = await ensureGameListVisible(page);
		await gameList.getByText('Catan').first().click();

		// Open sessions (mobile: drawer; desktop: sidebar)
		const sessionArea = await ensureSessionsVisible(page);
		await expect(sessionArea.getByText('Chat about Catan').first()).toBeVisible({
			timeout: 5_000
		});
		await sessionArea.locator('button').filter({ hasText: 'Chat about Catan' }).last().click();

		// Chat messages should appear
		await expect(page.getByText('How do I win at Catan?')).toBeVisible({ timeout: 5_000 });
		await expect(page.getByText(/10 victory points/)).toBeVisible();
	});

	test('send message input is visible in active session', async ({ page }) => {
		const gameList = await ensureGameListVisible(page);
		await gameList.getByText('Catan').first().click();

		const sessionArea = await ensureSessionsVisible(page);
		await sessionArea.locator('button').filter({ hasText: 'Chat about Catan' }).last().click();

		await expect(page.getByPlaceholder(/ask about game rules/i)).toBeVisible({ timeout: 5_000 });
	});

	test('session list in sidebar', async ({ page }) => {
		const gameList = await ensureGameListVisible(page);
		await gameList.getByText('Catan').first().click();

		const sessionArea = await ensureSessionsVisible(page);
		await expect(sessionArea.getByText(/chat sessions/i).first()).toBeVisible({
			timeout: 5_000
		});
		await expect(sessionArea.getByRole('button', { name: /new chat/i }).first()).toBeVisible();
	});

	test('house rules toggle in chat header', async ({ page }) => {
		const gameList = await ensureGameListVisible(page);
		await gameList.getByText('Catan').first().click();

		const sessionArea = await ensureSessionsVisible(page);
		await sessionArea.locator('button').filter({ hasText: 'Chat about Catan' }).last().click();

		// On mobile (< sm breakpoint), the "House Rules" label is hidden but the toggle switch exists
		if (isMobile(page)) {
			await expect(page.getByRole('switch').first()).toBeVisible({ timeout: 5_000 });
		} else {
			await expect(page.getByText(/house rules/i).first()).toBeVisible({ timeout: 5_000 });
		}
	});
});
