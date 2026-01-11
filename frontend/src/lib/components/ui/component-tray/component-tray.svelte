<script lang="ts" module>
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	export type ComponentTrayProps = HTMLAttributes<HTMLDivElement> & {
		title?: string;
	};
</script>

<script lang="ts">
	let {
		class: className,
		title,
		children,
		...restProps
	}: ComponentTrayProps = $props();
</script>

<div class={cn('component-tray', className)} {...restProps}>
	{#if title}
		<h4 class="font-display text-parchment text-sm font-semibold uppercase tracking-wide mb-3">{title}</h4>
	{/if}
	{@render children?.()}
</div>
