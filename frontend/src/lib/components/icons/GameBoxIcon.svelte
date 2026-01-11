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
	<!-- Box bottom -->
	<path d="M3 9L12 4L21 9V18L12 23L3 18V9Z" fill="var(--wood-light, #d4a574)" stroke="currentColor" />
	<!-- Box lid slightly open -->
	<path d="M3 9L12 4L21 9" fill="none" stroke="currentColor" stroke-width="2" />
	<path d="M12 4V2" stroke="currentColor" stroke-width="1.5" />
	<!-- Box top edge -->
	<path d="M3 9L12 14L21 9" stroke="var(--wood-dark, #8b6914)" stroke-width="1" />
	<!-- Center line -->
	<path d="M12 14V23" stroke="var(--wood-dark, #8b6914)" stroke-width="1" />
	<!-- Decorative element on top -->
	<circle cx="12" cy="14" r="2" fill="var(--gold-foil, #d4af37)" stroke="none" />
</svg>
