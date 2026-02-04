<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';

	// Redirect to the new tabbed games page with custom tab

	$effect(() => {
		if (browser) {
			goto(resolve('/games') + '?tab=custom', { replaceState: true });
		}
	});
</script>

<div class="flex min-h-[50vh] items-center justify-center">
	<p class="text-muted-foreground">Redirecting to games...</p>
</div>
