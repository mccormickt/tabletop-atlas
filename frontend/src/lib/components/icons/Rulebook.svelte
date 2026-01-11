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
	<!-- Book cover -->
	<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
	<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" fill="var(--wood-light, #d4a574)" />
	<!-- Book spine -->
	<path d="M4 4.5V19.5" stroke="var(--wood-dark, #8b6914)" stroke-width="3" />
	<!-- Page lines -->
	<line x1="9" y1="7" x2="16" y2="7" stroke="currentColor" stroke-width="1" />
	<line x1="9" y1="10" x2="16" y2="10" stroke="currentColor" stroke-width="1" />
	<line x1="9" y1="13" x2="14" y2="13" stroke="currentColor" stroke-width="1" />
	<!-- Bookmark -->
	<path d="M15 2V6L17 4.5L19 6V2" fill="var(--game-red, #dc2626)" stroke="none" />
</svg>
