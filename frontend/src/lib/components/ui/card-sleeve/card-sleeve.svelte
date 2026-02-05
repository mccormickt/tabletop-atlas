<script lang="ts" module>
	import { cn } from '$lib/utils.js';
	import type { HTMLAttributes } from 'svelte/elements';
	import { type VariantProps, tv } from 'tailwind-variants';

	export const cardSleeveVariants = tv({
		base: 'card-sleeve p-4 cursor-pointer',
		variants: {
			variant: {
				default: '',
				stacked: 'transform-none hover:transform-none',
				highlighted: 'border-gold-foil'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type CardSleeveVariant = VariantProps<typeof cardSleeveVariants>['variant'];

	export type CardSleeveProps = HTMLAttributes<HTMLElement> & {
		variant?: CardSleeveVariant;
		href?: string;
	};
</script>

<script lang="ts">
	import { resolve } from '$app/paths';

	let {
		class: className,
		variant = 'default',
		href,
		children,
		...restProps
	}: CardSleeveProps = $props();
</script>

{#if href}
	<a
		href={resolve(href as '/')}
		class={cn(cardSleeveVariants({ variant }), 'block no-underline', className)}
		{...restProps}
	>
		{@render children?.()}
	</a>
{:else}
	<div class={cn(cardSleeveVariants({ variant }), className)} {...restProps}>
		{@render children?.()}
	</div>
{/if}
