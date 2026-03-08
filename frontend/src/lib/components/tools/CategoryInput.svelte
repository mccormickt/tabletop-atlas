<script lang="ts">
	import IntegerInput from './inputs/IntegerInput.svelte';
	import CounterInput from './inputs/CounterInput.svelte';
	import SelectInput from './inputs/SelectInput.svelte';
	import ScienceInput from './inputs/ScienceInput.svelte';
	import type { ScoringCategory } from '$lib';

	let {
		category,
		value,
		onChange,
		onScienceChange
	}: {
		category: ScoringCategory;
		value: number;
		onChange: (value: number) => void;
		onScienceChange?: (tablets: number, compasses: number, gears: number) => void;
	} = $props();
</script>

{#if category.inputType === 'integer'}
	<IntegerInput
		{value}
		min={category.min ?? undefined}
		max={category.max ?? undefined}
		step={category.step ?? 1}
		{onChange}
	/>
{:else if category.inputType === 'counter'}
	<CounterInput
		{value}
		min={category.min ?? 0}
		max={category.max ?? undefined}
		step={category.step ?? 1}
		{onChange}
	/>
{:else if typeof category.inputType === 'object' && 'select' in category.inputType}
	<SelectInput options={category.inputType.select.options} {value} {onChange} />
{:else if category.inputType === 'science_symbols'}
	<ScienceInput {onScienceChange} />
{:else}
	<!-- Fallback for unknown types -->
	<IntegerInput
		{value}
		min={category.min ?? undefined}
		max={category.max ?? undefined}
		step={category.step ?? 1}
		{onChange}
	/>
{/if}
