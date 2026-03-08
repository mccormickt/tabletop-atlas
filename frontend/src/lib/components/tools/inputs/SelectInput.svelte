<script lang="ts">
	let {
		options,
		value,
		onChange
	}: {
		options: { value: number; label: string }[];
		value: number;
		onChange: (value: number) => void;
	} = $props();

	function handleChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		onChange(parseInt(target.value, 10));
	}
</script>

<select
	class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-center text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none"
	{value}
	onchange={handleChange}
>
	{#each options as option (option.value)}
		<option value={option.value}>{option.label}</option>
	{/each}
</select>
