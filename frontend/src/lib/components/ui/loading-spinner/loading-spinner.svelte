<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const loadingSpinnerVariants = tv({
		base: 'animate-spin rounded-full border-b-2',
		variants: {
			size: {
				xs: 'h-3 w-3',
				sm: 'h-4 w-4',
				default: 'h-6 w-6',
				lg: 'h-8 w-8',
				xl: 'h-12 w-12'
			},
			variant: {
				default: 'border-primary',
				muted: 'border-muted-foreground',
				accent: 'border-accent-foreground',
				destructive: 'border-destructive'
			}
		},
		defaultVariants: {
			size: 'default',
			variant: 'default'
		}
	});

	export type LoadingSpinnerSize = VariantProps<typeof loadingSpinnerVariants>['size'];
	export type LoadingSpinnerVariant = VariantProps<typeof loadingSpinnerVariants>['variant'];
</script>

<script lang="ts">
	import { cn } from '$lib/utils.js';

	interface LoadingSpinnerProps {
		size?: LoadingSpinnerSize;
		variant?: LoadingSpinnerVariant;
		text?: string;
		class?: string;
	}

	let {
		size = 'default',
		variant = 'default',
		text,
		class: className,
		...restProps
	}: LoadingSpinnerProps = $props();
</script>

{#if text}
	<div class="flex items-center justify-center space-x-2" {...restProps}>
		<div class={cn(loadingSpinnerVariants({ size, variant }), className)}></div>
		<span class="text-muted-foreground text-sm">{text}</span>
	</div>
{:else}
	<div class={cn(loadingSpinnerVariants({ size, variant }), className)} {...restProps}></div>
{/if}
