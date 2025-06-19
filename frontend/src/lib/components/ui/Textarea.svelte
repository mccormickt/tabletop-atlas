<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLTextareaAttributes } from 'svelte/elements';

	interface TextareaProps extends HTMLTextareaAttributes {
		class?: string;
		error?: boolean;
	}

	let {
		class: className = '',
		error = false,
		...restProps
	}: TextareaProps = $props();
</script>

<textarea
	class={cn(
		'flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
		error && 'border-destructive focus-visible:ring-destructive',
		className
	)}
	{...restProps}
></textarea>
