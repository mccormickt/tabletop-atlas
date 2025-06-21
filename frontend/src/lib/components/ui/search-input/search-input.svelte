<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const searchInputVariants = tv({
		base: 'border-input bg-background selection:bg-primary dark:bg-input/30 selection:text-primary-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive flex h-9 w-full min-w-0 rounded-md border px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
		variants: {
			size: {
				default: 'h-9',
				sm: 'h-8 text-sm',
				lg: 'h-10 px-4'
			}
		},
		defaultVariants: {
			size: 'default'
		}
	});

	export type SearchInputSize = VariantProps<typeof searchInputVariants>['size'];
</script>

<script lang="ts">
	import { cn, type WithElementRef } from '$lib/utils.js';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { createEventDispatcher } from 'svelte';

	interface SearchInputProps {
		ref?: HTMLInputElement | null;
		value?: string;
		size?: SearchInputSize;
		loading?: boolean;
		showClearButton?: boolean;
		keyboardShortcut?: string;
		onSearch?: (query: string) => void;
		onClear?: () => void;
	}

	let {
		ref = $bindable(null),
		value = $bindable(''),
		size = 'default',
		loading = false,
		showClearButton = true,
		keyboardShortcut,
		onSearch,
		onClear,
		class: className,
		placeholder = 'Search...',
		disabled = false,
		...restProps
	}: SearchInputProps &
		WithElementRef<HTMLInputElement> &
		Omit<HTMLInputAttributes, keyof SearchInputProps> = $props();

	const dispatch = createEventDispatcher<{
		search: string;
		clear: void;
		input: string;
	}>();

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;
		dispatch('input', value);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			handleSearch();
		}
	}

	function handleSearch() {
		if (value.trim()) {
			onSearch?.(value.trim());
			dispatch('search', value.trim());
		}
	}

	function handleClear() {
		value = '';
		onClear?.();
		dispatch('clear');
		ref?.focus();
	}

	function handleSearchButtonClick() {
		handleSearch();
	}
</script>

<div class="relative flex items-center">
	<!-- Search Icon -->
	<div class="pointer-events-none absolute left-3 flex items-center">
		{#if loading}
			<svg class="text-muted-foreground h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
				></circle>
				<path
					class="opacity-75"
					fill="currentColor"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
				></path>
			</svg>
		{:else}
			<svg
				class="text-muted-foreground h-4 w-4"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
				></path>
			</svg>
		{/if}
	</div>

	<!-- Input Field -->
	<input
		bind:this={ref}
		{value}
		{placeholder}
		{disabled}
		class={cn(
			searchInputVariants({ size }),
			'pl-10',
			(showClearButton && value) || keyboardShortcut ? 'pr-20' : 'pr-3',
			className
		)}
		oninput={handleInput}
		onkeydown={handleKeydown}
		{...restProps}
	/>

	<!-- Right Side Controls -->
	<div class="absolute right-2 flex items-center space-x-1">
		<!-- Clear Button -->
		{#if showClearButton && value && !loading}
			<button
				type="button"
				onclick={handleClear}
				class="text-muted-foreground hover:text-foreground flex h-5 w-5 items-center justify-center rounded transition-colors"
				aria-label="Clear search"
			>
				<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M6 18L18 6M6 6l12 12"
					></path>
				</svg>
			</button>
		{/if}

		<!-- Keyboard Shortcut Display -->
		{#if keyboardShortcut && !value}
			<div class="hidden items-center sm:flex">
				<kbd
					class="text-muted-foreground bg-muted border-border rounded border px-2 py-0.5 text-xs"
				>
					{keyboardShortcut}
				</kbd>
			</div>
		{/if}
	</div>
</div>
