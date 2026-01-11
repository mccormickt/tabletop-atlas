<script lang="ts" module>
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	export type ComponentTraySectionProps = HTMLAttributes<HTMLDivElement>;
</script>

<script lang="ts">
	let {
		class: className,
		children,
		...restProps
	}: ComponentTraySectionProps = $props();
</script>

<div class={cn('component-tray-section', className)} {...restProps}>
	{@render children?.()}
</div>
