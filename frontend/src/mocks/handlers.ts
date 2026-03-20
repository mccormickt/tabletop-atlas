import { http, HttpResponse } from 'msw';
import * as data from './data';

export const handlers = [
	// --- Auth ---
	http.get('/api/auth/me', () => HttpResponse.json({ user: data.mockUser })),
	http.post('/api/auth/refresh', () => HttpResponse.json({})),
	http.post('/api/auth/logout', () => HttpResponse.json({})),

	// --- Games ---
	http.get('/api/games', ({ request }) => {
		const url = new URL(request.url);
		const search = url.searchParams.get('search');
		if (search) return HttpResponse.json(data.mockFilteredGamesPage(search));
		return HttpResponse.json(data.mockGamesPage);
	}),
	http.get('/api/games/:id', () => HttpResponse.json(data.mockGameDetail)),
	http.post('/api/games', () => HttpResponse.json(data.mockGameDetail, { status: 201 })),
	http.put('/api/games/:id', () => HttpResponse.json(data.mockGameDetail)),
	http.delete('/api/games/:id', () => new HttpResponse(null, { status: 204 })),

	// --- Rules Info ---
	http.get('/api/games/:id/rules-info', ({ params }) => {
		const id = Number(params.id);
		if (id === 1) return HttpResponse.json(data.mockRulesInfo);
		return HttpResponse.json(data.mockNoRulesInfo);
	}),
	http.post('/api/games/:id/rules-upload', () =>
		HttpResponse.json({
			message: 'Rules uploaded successfully',
			file_path: '/rules/test.pdf',
			text_length: 5000,
			chunks_processed: 20
		})
	),
	http.delete('/api/games/:id/rules', () =>
		HttpResponse.json({
			message: 'Rules deleted',
			file_deleted: true,
			embeddings_deleted: 20
		})
	),

	// --- House Rules ---
	http.get('/api/house-rules', () => HttpResponse.json(data.mockHouseRulesPage)),
	http.post('/api/house-rules', () =>
		HttpResponse.json(data.mockHouseRulesPage.items[0], { status: 201 })
	),
	http.patch('/api/house-rules/:id', () => HttpResponse.json(data.mockHouseRulesPage.items[0])),
	http.delete('/api/house-rules/:id', () => new HttpResponse(null, { status: 204 })),

	// --- Chat ---
	http.get('/api/chat/sessions', () => HttpResponse.json(data.mockChatSessions)),
	http.post('/api/chat/sessions', () => HttpResponse.json(data.mockNewSession, { status: 201 })),
	http.get('/api/chat/sessions/:id', () => HttpResponse.json(data.mockChatHistory)),
	http.put('/api/chat/sessions/:id', () => HttpResponse.json(data.mockChatHistory.session)),
	http.post('/api/chat', () => HttpResponse.json(data.mockChatResponse)),

	// --- Search ---
	http.get('/api/chat/search-rules', () => HttpResponse.json(data.mockSearchResults)),

	// --- Collection ---
	http.get('/api/collection', () => HttpResponse.json(data.mockCollectionPage)),
	http.post('/api/collection', () =>
		HttpResponse.json(data.mockCollectionPage.items[0], { status: 201 })
	),
	http.delete('/api/collection/:id', () => new HttpResponse(null, { status: 204 })),

	// --- Custom Games ---
	http.get('/api/custom-games', () => HttpResponse.json(data.mockCustomGamesPage)),
	http.post('/api/custom-games', () =>
		HttpResponse.json(data.mockCustomGamesPage.items[0], { status: 201 })
	),
	http.delete('/api/custom-games/:id', () => new HttpResponse(null, { status: 204 })),

	// --- Challenges ---
	http.get('/api/challenges', () => HttpResponse.json(data.mockChallengesPage)),
	http.get('/api/challenges/:id', () => HttpResponse.json(data.mockChallengeDetail)),
	http.get('/api/challenges/:id/grid', () => HttpResponse.json(data.mockChallengeGridView)),
	http.get('/api/challenges/:id/stats', () => HttpResponse.json(data.mockChallengeStats)),
	http.post('/api/challenges', () => HttpResponse.json(data.mockChallengeDetail, { status: 201 })),
	http.post('/api/challenges/:id/participants', () => HttpResponse.json({}, { status: 201 })),
	http.post('/api/challenges/:id/games', () => HttpResponse.json({}, { status: 201 })),
	http.post('/api/challenges/:id/plays', () => HttpResponse.json({}, { status: 201 })),

	// --- Tools ---
	http.get('/api/tools', () => HttpResponse.json(data.mockToolsList)),
	http.get('/api/tools/:toolId', () => HttpResponse.json(data.mockToolDetail)),
	http.post('/api/tools/:toolId/calculate', () => HttpResponse.json(data.mockScoreOutput)),

	// --- Admin ---
	http.get('/api/admin/stats', () => HttpResponse.json(data.mockAdminStats)),
	http.get('/api/admin/bgg/stats', () => HttpResponse.json(data.mockEnrichmentStats)),
	http.get('/api/admin/users', () => HttpResponse.json(data.mockUsersPage)),
	http.put('/api/admin/users/:id/role', () => HttpResponse.json(data.mockUsersPage.items[0]))
];
