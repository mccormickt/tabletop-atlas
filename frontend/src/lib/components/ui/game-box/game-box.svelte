<script lang="ts" module>
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';
	import { type VariantProps, tv } from 'tailwind-variants';

	export const gameBoxVariants = tv({
		base: 'game-box-lid p-6 relative',
		variants: {
			variant: {
				default: '',
				featured: 'border-gold-foil',
				muted: 'opacity-90'
			},
			size: {
				default: 'p-6',
				sm: 'p-4',
				lg: 'p-8'
			}
		},
		defaultVariants: {
			variant: 'default',
			size: 'default'
		}
	});

	export type GameBoxVariant = VariantProps<typeof gameBoxVariants>['variant'];
	export type GameBoxSize = VariantProps<typeof gameBoxVariants>['size'];

	export type GameBoxProps = HTMLAttributes<HTMLDivElement> & {
		variant?: GameBoxVariant;
		size?: GameBoxSize;
		title?: string;
		showCorners?: boolean;
	};
</script>

<script lang="ts">
	let {
		class: className,
		variant = 'default',
		size = 'default',
		title,
		showCorners = false,
		children,
		...restProps
	}: GameBoxProps = $props();
</script>

<div
	class={cn(
		gameBoxVariants({ variant, size }),
		showCorners && 'decorative-corners',
		className
	)}
	{...restProps}
>
	{#if title}
		<h3 class="rulebook-header text-xl">{title}</h3>
	{/if}
	{@render children?.()}
</div>
