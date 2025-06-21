<script lang="ts" module>
	import { type VariantProps, tv } from "tailwind-variants";

	export const emptyStateVariants = tv({
		base: "flex flex-col items-center justify-center text-center",
		variants: {
			size: {
				sm: "py-8 px-4",
				default: "py-12 px-6",
				lg: "py-16 px-8"
			}
		},
		defaultVariants: {
			size: "default"
		}
	});

	export type EmptyStateSize = VariantProps<typeof emptyStateVariants>["size"];
</script>

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import { Button } from "$lib/components/ui/button";

	interface EmptyStateProps {
		icon?: "search" | "document" | "game" | "upload" | "chat" | "custom";
		title: string;
		description?: string;
		actionText?: string;
		secondaryActionText?: string;
		size?: EmptyStateSize;
		class?: string;
		onAction?: () => void;
		onSecondaryAction?: () => void;
	}

	let {
		icon = "search",
		title,
		description,
		actionText,
		secondaryActionText,
		size = "default",
		class: className,
		onAction,
		onSecondaryAction,
		children
	}: EmptyStateProps & { children?: any } = $props();

	function getIconSvg(iconType: string) {
		const iconSize = size === "lg" ? "h-16 w-16" : size === "sm" ? "h-8 w-8" : "h-12 w-12";
		const iconClass = `mx-auto ${iconSize} text-gray-400`;

		switch (iconType) {
			case "search":
				return `<svg class="${iconClass}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
				</svg>`;
			case "document":
				return `<svg class="${iconClass}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
				</svg>`;
			case "game":
				return `<svg class="${iconClass}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
				</svg>`;
			case "upload":
				return `<svg class="${iconClass}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
				</svg>`;
			case "chat":
				return `<svg class="${iconClass}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
				</svg>`;
			default:
				return "";
		}
	}
</script>

<div class={cn(emptyStateVariants({ size }), className)}>
	<!-- Icon -->
	{#if icon && icon !== "custom"}
		<div class="mb-4">
			{@html getIconSvg(icon)}
		</div>
	{:else if children?.icon}
		<div class="mb-4 text-gray-400">
			{@render children.icon()}
		</div>
	{/if}

	<!-- Title -->
	<h3 class="text-lg font-medium text-gray-900 {size === 'lg' ? 'text-xl' : size === 'sm' ? 'text-base' : 'text-lg'}">
		{title}
	</h3>

	<!-- Description -->
	{#if description}
		<p class="mt-2 text-gray-600 {size === 'lg' ? 'text-base max-w-md' : size === 'sm' ? 'text-sm max-w-xs' : 'text-sm max-w-sm'}">
			{description}
		</p>
	{/if}

	<!-- Custom content -->
	{#if children?.content}
		<div class="mt-4">
			{@render children.content()}
		</div>
	{/if}

	<!-- Actions -->
	{#if actionText || secondaryActionText || children?.actions}
		<div class="mt-6 flex flex-col sm:flex-row gap-3 items-center justify-center">
			{#if actionText && onAction}
				<Button onclick={onAction} size={size === "sm" ? "sm" : "default"}>
					{actionText}
				</Button>
			{/if}

			{#if secondaryActionText && onSecondaryAction}
				<Button variant="outline" onclick={onSecondaryAction} size={size === "sm" ? "sm" : "default"}>
					{secondaryActionText}
				</Button>
			{/if}

			{#if children?.actions}
				{@render children.actions()}
			{/if}
		</div>
	{/if}
</div>
