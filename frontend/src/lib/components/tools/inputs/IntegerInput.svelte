<script lang="ts">
	import { Input } from '$lib/components/ui/input';

	let {
		value,
		min,
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

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		let newValue = parseInt(target.value, 10);
		if (isNaN(newValue)) newValue = 0;
		if (min !== undefined && newValue < min) newValue = min;
		if (max !== undefined && newValue > max) newValue = max;
		onChange(newValue);
	}
</script>

<Input type="number" {value} {min} {max} {step} class="w-full text-center" oninput={handleInput} />
