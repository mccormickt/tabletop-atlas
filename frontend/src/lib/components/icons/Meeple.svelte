<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { SVGAttributes } from 'svelte/elements';

	type Props = SVGAttributes<SVGSVGElement> & {
		size?: number | string;
		color?: 'red' | 'blue' | 'green' | 'yellow' | 'orange' | 'purple' | 'current';
	};

	let {
		size = 24,
		color = 'current',
		class: className,
		...restProps
	}: Props = $props();

	const colorMap = {
		red: 'var(--game-red)',
		blue: 'var(--game-blue)',
		green: 'var(--game-green)',
		yellow: 'var(--game-yellow)',
		orange: 'var(--game-orange)',
		purple: 'var(--game-purple)',
		current: 'currentColor'
	};
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill={colorMap[color]}
	class={cn('inline-block', className)}
	aria-hidden="true"
	{...restProps}
>
	<!-- Meeple shape -->
	<path d="M12 2C10.5 2 9.5 3 9.5 4.5C9.5 5.5 10 6.3 10.7 6.7C9.3 7.5 8 9 8 11L6 11C5.4 11 5 11.4 5 12L5 21C5 21.6 5.4 22 6 22L9 22L9 16L15 16L15 22L18 22C18.6 22 19 21.6 19 21L19 12C19 11.4 18.6 11 18 11L16 11C16 9 14.7 7.5 13.3 6.7C14 6.3 14.5 5.5 14.5 4.5C14.5 3 13.5 2 12 2Z" />
</svg>
