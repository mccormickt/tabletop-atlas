<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { SVGAttributes } from 'svelte/elements';

	type Props = SVGAttributes<SVGSVGElement> & {
		size?: number | string;
		value?: 1 | 2 | 3 | 4 | 5 | 6;
	};

	let { size = 24, value = 6, class: className, ...restProps }: Props = $props();

	// Dot positions for each dice value
	const dotPositions: Record<number, Array<{ cx: number; cy: number }>> = {
		1: [{ cx: 12, cy: 12 }],
		2: [
			{ cx: 8, cy: 8 },
			{ cx: 16, cy: 16 }
		],
		3: [
			{ cx: 8, cy: 8 },
			{ cx: 12, cy: 12 },
			{ cx: 16, cy: 16 }
		],
		4: [
			{ cx: 8, cy: 8 },
			{ cx: 16, cy: 8 },
			{ cx: 8, cy: 16 },
			{ cx: 16, cy: 16 }
		],
		5: [
			{ cx: 8, cy: 8 },
			{ cx: 16, cy: 8 },
			{ cx: 12, cy: 12 },
			{ cx: 8, cy: 16 },
			{ cx: 16, cy: 16 }
		],
		6: [
			{ cx: 8, cy: 7 },
			{ cx: 16, cy: 7 },
			{ cx: 8, cy: 12 },
			{ cx: 16, cy: 12 },
			{ cx: 8, cy: 17 },
			{ cx: 16, cy: 17 }
		]
	};
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	class={cn('inline-block', className)}
	aria-hidden="true"
	{...restProps}
>
	<!-- Dice body -->
	<rect
		x="2"
		y="2"
		width="20"
		height="20"
		rx="3"
		fill="white"
		stroke="currentColor"
		stroke-width="2"
	/>
	<!-- Dots -->
	{#each dotPositions[value] || [] as dot, i (i)}
		<circle cx={dot.cx} cy={dot.cy} r="2" fill="currentColor" />
	{/each}
</svg>
