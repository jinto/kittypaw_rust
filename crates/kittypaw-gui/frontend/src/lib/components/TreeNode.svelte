<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { FileEntry } from '$lib/stores/workspace';

	export let node: { entry: FileEntry; children: { entry: FileEntry; children: any[] }[] };
	export let depth: number = 0;
	export let expandedDirs: Set<string>;
	export let selectedFile: string | null;

	const dispatch = createEventDispatcher<{ toggle: string; select: string }>();

	function getFileName(path: string): string {
		return path.split('/').pop() ?? path;
	}
</script>

<div class="tree-item" style="padding-left: {depth * 14 + 8}px">
	{#if node.entry.is_dir}
		<button
			class="tree-btn dir"
			on:click={() => dispatch('toggle', node.entry.path)}
			title={node.entry.path}
		>
			<span class="arrow" class:open={expandedDirs.has(node.entry.path)}>▶</span>
			<span class="icon">📁</span>
			<span class="name">{getFileName(node.entry.path)}</span>
		</button>
		{#if expandedDirs.has(node.entry.path)}
			{#each node.children as child}
				<svelte:self
					node={child}
					depth={depth + 1}
					{expandedDirs}
					{selectedFile}
					on:toggle
					on:select
				/>
			{/each}
		{/if}
	{:else}
		<button
			class="tree-btn file"
			class:active={selectedFile === node.entry.path}
			on:click={() => dispatch('select', node.entry.path)}
			title={node.entry.path}
		>
			<span class="icon">📄</span>
			<span class="name">{getFileName(node.entry.path)}</span>
		</button>
	{/if}
</div>

<style>
	.tree-item {
		display: flex;
		flex-direction: column;
	}

	.tree-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		width: 100%;
		border: none;
		background: transparent;
		color: #94a3b8;
		font-size: 12px;
		cursor: pointer;
		padding: 4px 8px;
		text-align: left;
		border-radius: 4px;
		transition: background 0.1s, color 0.1s;
		white-space: nowrap;
		overflow: hidden;
	}

	.tree-btn:hover {
		background: #1e293b;
		color: #e2e8f0;
	}

	.tree-btn.active {
		background: #1e3a5f;
		color: #93c5fd;
	}

	.arrow {
		font-size: 8px;
		display: inline-block;
		transition: transform 0.15s;
		color: #475569;
		width: 10px;
		flex-shrink: 0;
	}

	.arrow.open {
		transform: rotate(90deg);
	}

	.icon {
		flex-shrink: 0;
	}

	.name {
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
