<script lang="ts">
	import { cn } from '$lib/utils';

	let {
		id,
		checked = false,
		disabled = false,
		onCheckedChange,
		class: className
	}: {
		id?: string;
		checked?: boolean;
		disabled?: boolean;
		onCheckedChange?: (checked: boolean) => void;
		class?: string;
	} = $props();
</script>

<button
	type="button"
	role="checkbox"
	aria-checked={checked}
	{id}
	{disabled}
	class={cn(
		'border-primary ring-offset-background focus-visible:ring-ring data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground peer h-4 w-4 shrink-0 rounded-sm border focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
	data-state={checked ? 'checked' : 'unchecked'}
	onclick={() => !disabled && onCheckedChange?.(!checked)}
>
	{#if checked}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="3"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="h-4 w-4"
		>
			<polyline points="20 6 9 17 4 12" />
		</svg>
	{/if}
</button>
