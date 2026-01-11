<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { SVGAttributes } from 'svelte/elements';

	type Props = SVGAttributes<SVGSVGElement> & {
		size?: number | string;
	};

	let {
		size = 24,
		class: className,
		...restProps
	}: Props = $props();
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
	<circle cx="11" cy="11" r="8" />
	<path d="M21 21L16.65 16.65" />
</svg>
