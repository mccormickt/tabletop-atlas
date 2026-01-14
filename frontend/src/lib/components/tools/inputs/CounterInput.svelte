<script lang="ts">
	import { Button } from '$lib/components/ui/button';

	let {
		value,
		min = 0,
		max,
		step = 1,
		onChange
	}: {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		onChange: (value: number) => void;
	} = $props();

	function decrement() {
		const newValue = value - step;
		if (min === undefined || newValue >= min) {
			onChange(newValue);
		}
	}

	function increment() {
		const newValue = value + step;
		if (max === undefined || newValue <= max) {
			onChange(newValue);
		}
	}
</script>

<div class="flex items-center justify-center gap-2">
	<Button
		variant="outline"
		size="sm"
		disabled={min !== undefined && value <= min}
		onclick={decrement}
	>
		-
	</Button>
	<span class="text-foreground w-8 text-center font-semibold">{value}</span>
	<Button
		variant="outline"
		size="sm"
		disabled={max !== undefined && value >= max}
		onclick={increment}
	>
		+
	</Button>
</div>
