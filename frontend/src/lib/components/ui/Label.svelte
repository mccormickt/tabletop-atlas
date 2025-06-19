<script lang="ts">
	import { cn } from '$lib/utils';
	import type { HTMLLabelAttributes } from 'svelte/elements';

	interface LabelProps extends HTMLLabelAttributes {
		class?: string;
	}

	let {
		class: className = '',
		children,
		...restProps
	}: LabelProps = $props();
</script>

<label
	class={cn(
		'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
		className
	)}
	{...restProps}
>
	{@render children?.()}
</label>
