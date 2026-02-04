import { describe, test, expect, vi } from 'vitest';
import '@testing-library/jest-dom/vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';

// Mock fetch to prevent unhandled rejections from API calls during render
vi.stubGlobal(
	'fetch',
	vi.fn().mockResolvedValue(
		new Response(JSON.stringify({ items: [], total: 0, page: 1, totalPages: 1 }), {
			status: 200,
			headers: { 'Content-Type': 'application/json' }
		})
	)
);

describe('/+page.svelte', () => {
	test('should render h1', () => {
		render(Page);
		expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
	});
});
