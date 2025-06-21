<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const searchResultVariants = tv({
		base: 'w-full text-left border border-gray-200 rounded-lg p-4 space-y-3 transition-colors',
		variants: {
			variant: {
				default: 'hover:bg-gray-50',
				compact: 'p-3 space-y-2',
				highlighted: 'border-blue-300 bg-blue-50 hover:bg-blue-100'
			},
			interactive: {
				true: 'cursor-pointer',
				false: 'cursor-default'
			}
		},
		defaultVariants: {
			variant: 'default',
			interactive: true
		}
	});

	export type SearchResultVariant = VariantProps<typeof searchResultVariants>['variant'];
</script>

<script lang="ts">
	import { cn } from '$lib/utils.js';
	import { Badge } from '$lib/components/ui/badge';
	import { createEventDispatcher } from 'svelte';

	interface SearchResultProps {
		chunkId: string | number;
		chunkText: string;
		similarityScore: number;
		metadata?: string;
		index?: number;
		variant?: SearchResultVariant;
		interactive?: boolean;
		showMetadata?: boolean;
		showIndex?: boolean;
		showSimilarity?: boolean;
		maxTextLength?: number;
		class?: string;
	}

	let {
		chunkId,
		chunkText,
		similarityScore,
		metadata,
		index,
		variant = 'default',
		interactive = true,
		showMetadata = true,
		showIndex = true,
		showSimilarity = true,
		maxTextLength = 180,
		class: className,
		...restProps
	}: SearchResultProps = $props();

	const dispatch = createEventDispatcher<{
		click: {
			chunkId: string | number;
			chunkText: string;
			similarityScore: number;
			metadata?: string;
		};
	}>();

	function formatSimilarityScore(score: number): string {
		return (score * 100).toFixed(1) + '%';
	}

	function getSimilarityBadgeVariant(score: number): 'default' | 'secondary' | 'outline' {
		if (score >= 0.8) return 'default';
		if (score >= 0.6) return 'secondary';
		return 'outline';
	}

	function getSimilarityColor(score: number): string {
		if (score >= 0.8) return 'text-green-600';
		if (score >= 0.6) return 'text-yellow-600';
		return 'text-gray-600';
	}

	function truncateText(text: string, maxLength: number): string {
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
	}

	function handleClick() {
		if (interactive) {
			dispatch('click', {
				chunkId,
				chunkText,
				similarityScore,
				metadata
			});
		}
	}

	const displayText = $derived(
		variant === 'compact' && maxTextLength < 180
			? truncateText(chunkText, Math.min(maxTextLength, 120))
			: truncateText(chunkText, maxTextLength)
	);
</script>

<svelte:element
	this={interactive ? 'button' : 'div'}
	class={cn(searchResultVariants({ variant, interactive }), className)}
	onclick={handleClick}
	{...restProps}
>
	<!-- Header with badges and metadata -->
	<div class="flex items-start justify-between">
		<div class="flex items-center space-x-2">
			{#if showIndex && typeof index === 'number'}
				<Badge variant="outline" class="text-xs">
					#{index + 1}
				</Badge>
			{/if}

			{#if showSimilarity}
				<Badge
					variant={getSimilarityBadgeVariant(similarityScore)}
					class="text-xs {getSimilarityColor(similarityScore)}"
				>
					{formatSimilarityScore(similarityScore)} match
				</Badge>
			{/if}
		</div>

		{#if showMetadata && metadata}
			<div class="ml-2 truncate text-xs text-gray-500">
				{metadata}
			</div>
		{/if}
	</div>

	<!-- Content -->
	<div class="prose prose-sm max-w-none">
		<p class="leading-relaxed text-gray-900 {variant === 'compact' ? 'text-sm' : 'text-sm'}">
			{displayText}
		</p>
	</div>

	<!-- Footer with technical details (only shown in non-compact mode) -->
	{#if variant !== 'compact' && (chunkId || similarityScore)}
		<div class="flex items-center justify-between pt-1 text-xs text-gray-500">
			{#if chunkId}
				<span>Chunk ID: {String(chunkId)}</span>
			{/if}
			{#if similarityScore}
				<span>Similarity: {similarityScore.toFixed(3)}</span>
			{/if}
		</div>
	{/if}
</svelte:element>
