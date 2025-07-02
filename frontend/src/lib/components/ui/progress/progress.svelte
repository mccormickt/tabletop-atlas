<script lang="ts">
	import { cn } from '$lib/utils';

	let className: string | undefined = undefined;
	export { className as class };
	export let value: number = 0;
	export let max: number = 100;

	$: percentage = Math.min(Math.max((value / max) * 100, 0), 100);
</script>

<div
	class={cn('bg-secondary relative h-4 w-full overflow-hidden rounded-full', className)}
	{...$$restProps}
>
	<div
		class="bg-primary h-full w-full flex-1 transition-all"
		style="transform: translateX(-{100 - percentage}%)"
	></div>
</div>
