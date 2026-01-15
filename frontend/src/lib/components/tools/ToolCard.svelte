<script lang="ts">
	import type { ToolSummary } from '$api/Api';

	let { tool }: { tool: ToolSummary } = $props();

	const toolTypeInfo: Record<string, { label: string; icon: string }> = {
		score_calculator: { label: 'Score Calculator', icon: '🧮' },
		timer: { label: 'Timer', icon: '⏱️' },
		dice_roller: { label: 'Dice Roller', icon: '🎲' },
		randomizer: { label: 'Randomizer', icon: '🎰' }
	};

	function getToolInfo(type: string): { label: string; icon: string } {
		return toolTypeInfo[type] ?? { label: type, icon: '🔧' };
	}

	let toolInfo = $derived(getToolInfo(tool.toolType));
</script>

<a
	href="/tools/{tool.id}"
	class="bg-card border-border hover:border-primary group block rounded-lg border p-4 transition-all hover:shadow-md"
>
	<div class="mb-3 flex items-start justify-between">
		<div class="flex items-center gap-3">
			<span class="text-2xl">{toolInfo.icon}</span>
			<h3 class="text-foreground group-hover:text-primary font-semibold">{tool.displayName}</h3>
		</div>
	</div>

	<div class="flex items-center justify-between">
		<span class="bg-muted text-muted-foreground rounded-full px-2 py-1 text-xs font-medium">
			{toolInfo.label}
		</span>
		<span class="text-muted-foreground text-sm">
			{tool.playerRange.min}-{tool.playerRange.max} players
		</span>
	</div>
</a>
