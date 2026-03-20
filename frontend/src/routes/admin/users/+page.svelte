<script lang="ts">
	import { api, formatDate, type UserListItem } from '$lib';
	import { SearchInput, Badge, Pagination } from '$lib/components/ui';
	import { useAuth, type AuthState } from '$lib/stores/auth';

	let users = $state<UserListItem[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let page = $state(1);
	let totalPages = $state(0);
	let total = $state(0);

	let searchQuery = $state('');
	let roleFilter = $state('');
	let statusMessage = $state<{ text: string; type: 'success' | 'error' } | null>(null);
	let updatingUserId = $state<number | null>(null);

	const auth = useAuth();
	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
		});
		return unsubscribe;
	});

	let searchTimeout: ReturnType<typeof setTimeout>;
	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadUsers(1);
		}
	});

	async function loadUsers(pageNum: number) {
		isLoading = true;
		error = null;
		try {
			const result = await api.methods.listAdminUsers({
				query: {
					page: pageNum,
					limit: 20,
					search: searchQuery || undefined,
					role: roleFilter || undefined
				}
			});
			if (result.type === 'success') {
				users = result.data.items;
				page = result.data.page;
				totalPages = result.data.totalPages;
				total = result.data.total;
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to load users';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to load users';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load users';
		} finally {
			isLoading = false;
		}
	}

	function handleSearchInput(value: string) {
		searchQuery = value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			loadUsers(1);
		}, 300);
	}

	function handleSearchClear() {
		searchQuery = '';
		loadUsers(1);
	}

	function handleRoleFilterChange(event: Event) {
		roleFilter = (event.target as HTMLSelectElement).value;
		loadUsers(1);
	}

	function showStatus(text: string, type: 'success' | 'error') {
		statusMessage = { text, type };
		setTimeout(() => {
			statusMessage = null;
		}, 3000);
	}

	async function handleRoleChange(userId: number, newRole: string) {
		const previousUsers = [...users];
		updatingUserId = userId;

		// Optimistic update
		users = users.map((u) => (u.id === userId ? { ...u, role: newRole } : u));

		try {
			const result = await api.methods.updateUserRole({
				path: { id: userId },
				body: { role: newRole }
			});
			if (result.type === 'success') {
				users = users.map((u) => (u.id === userId ? result.data : u));
				showStatus('Role updated successfully', 'success');
			} else {
				users = previousUsers;
				let msg = 'Failed to update role';
				if (result.type === 'error') {
					msg = result.data.message || msg;
				} else if (result.type === 'client_error') {
					msg = result.error.message || msg;
				}
				showStatus(msg, 'error');
			}
		} catch (err) {
			users = previousUsers;
			showStatus(err instanceof Error ? err.message : 'Failed to update role', 'error');
		} finally {
			updatingUserId = null;
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Header -->
	<div class="mb-6">
		<h1 class="text-foreground text-3xl font-bold">User Management</h1>
		{#if !isLoading && !error}
			<p class="text-muted-foreground mt-1">{total} user{total !== 1 ? 's' : ''} total</p>
		{/if}
	</div>

	<!-- Status message -->
	{#if statusMessage}
		<div
			class="mb-4 rounded-lg border p-3 text-sm {statusMessage.type === 'success'
				? 'border-green-200 bg-green-50 text-green-800 dark:border-green-800 dark:bg-green-900/20 dark:text-green-200'
				: 'border-red-200 bg-red-50 text-red-800 dark:border-red-800 dark:bg-red-900/20 dark:text-red-200'}"
			role="status"
		>
			{statusMessage.text}
		</div>
	{/if}

	<!-- Filters -->
	<div class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center">
		<div class="flex-1">
			<SearchInput
				value={searchQuery}
				placeholder="Search by email or name..."
				onInput={handleSearchInput}
				onClear={handleSearchClear}
			/>
		</div>
		<select
			class="border-input bg-background text-foreground h-9 rounded-md border px-3 text-sm"
			value={roleFilter}
			onchange={handleRoleFilterChange}
		>
			<option value="">All Roles</option>
			<option value="admin">Admin</option>
			<option value="user">User</option>
		</select>
	</div>

	<!-- Content -->
	{#if isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
		</div>
	{:else if users.length === 0}
		<div class="text-muted-foreground py-12 text-center">
			<p>No users found.</p>
		</div>
	{:else}
		<div class="overflow-x-auto">
			<table class="w-full text-left text-sm">
				<thead>
					<tr class="border-border border-b">
						<th class="text-muted-foreground px-4 py-3 font-medium">Email</th>
						<th class="text-muted-foreground px-4 py-3 font-medium">Display Name</th>
						<th class="text-muted-foreground px-4 py-3 font-medium">Role</th>
						<th class="text-muted-foreground px-4 py-3 font-medium">Created</th>
					</tr>
				</thead>
				<tbody>
					{#each users as user (user.id)}
						<tr class="border-border hover:bg-muted/50 border-b transition-colors">
							<td class="text-foreground px-4 py-3">{user.email}</td>
							<td class="text-foreground px-4 py-3">
								{user.displayName || '-'}
							</td>
							<td class="px-4 py-3">
								{#if authState.user?.id === user.id}
									<div class="flex items-center gap-2">
										<Badge variant="outline">{user.role}</Badge>
										<span class="text-muted-foreground text-xs">(You)</span>
									</div>
								{:else}
									<div class="flex items-center gap-2">
										<select
											class="border-input bg-background text-foreground h-8 rounded-md border px-2 text-sm"
											value={user.role}
											disabled={updatingUserId === user.id}
											onchange={(e) =>
												handleRoleChange(user.id, (e.target as HTMLSelectElement).value)}
										>
											<option value="user">user</option>
											<option value="admin">admin</option>
										</select>
										{#if updatingUserId === user.id}
											<div
												class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
											></div>
										{/if}
									</div>
								{/if}
							</td>
							<td class="text-muted-foreground px-4 py-3">
								{formatDate(new Date(user.createdAt))}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if totalPages > 1}
			<div class="mt-6 flex justify-center">
				<Pagination currentPage={page} {totalPages} onPageChange={(p) => loadUsers(p)} />
			</div>
		{/if}
	{/if}
</div>
