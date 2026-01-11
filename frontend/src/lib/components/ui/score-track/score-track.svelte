<script lang="ts" module>
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';

	export type ScoreTrackProps = HTMLAttributes<HTMLDivElement> & {
		total: number;
		current: number;
		showNumbers?: boolean;
	};
</script>

<script lang="ts">
	let {
		class: className,
		total,
		current,
		showNumbers = true,
		...restProps
	}: ScoreTrackProps = $props();

	const segments = $derived(Array.from({ length: total }, (_, i) => i + 1));
</script>

<div
	class={cn('score-track', className)}
	role="progressbar"
	aria-valuenow={current}
	aria-valuemin={1}
	aria-valuemax={total}
	{...restProps}
>
	{#each segments as segment (segment)}
		<div
			class={cn(
				'score-track-segment',
				segment < current && 'completed',
				segment === current && 'active'
			)}
		>
			{#if showNumbers}
				{segment}
			{/if}
		</div>
	{/each}
</div>
