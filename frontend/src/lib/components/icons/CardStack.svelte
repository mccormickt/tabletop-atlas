<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { SVGAttributes } from 'svelte/elements';

	type Props = SVGAttributes<SVGSVGElement> & {
		size?: number | string;
	};

	let { size = 24, class: className, ...restProps }: Props = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	class={cn('inline-block', className)}
	aria-hidden="true"
	{...restProps}
>
	<!-- Bottom card -->
	<rect
		x="4"
		y="6"
		width="14"
		height="18"
		rx="2"
		fill="var(--parchment, #f5f5dc)"
		stroke="currentColor"
		transform="rotate(-8 11 15)"
	/>
	<!-- Middle card -->
	<rect
		x="5"
		y="4"
		width="14"
		height="18"
		rx="2"
		fill="var(--parchment, #f5f5dc)"
		stroke="currentColor"
		transform="rotate(-4 12 13)"
	/>
	<!-- Top card -->
	<rect x="6" y="2" width="14" height="18" rx="2" fill="white" stroke="currentColor" />
	<!-- Card decoration -->
	<path d="M10 8 L13 12 L10 16" stroke="currentColor" stroke-width="1.5" fill="none" />
</svg>
