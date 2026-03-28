<script lang="ts">
	import { fileTree, selectedFile, fileContent, currentWorkspace } from '$lib/stores/workspace';
	import type { FileEntry } from '$lib/stores/workspace';
	import { invoke } from '@tauri-apps/api/core';
	import TreeNode from './TreeNode.svelte';

	let searchQuery = '';
	let expandedDirs = new Set<string>();

	interface TreeNodeData {
		entry: FileEntry;
		children: TreeNodeData[];
	}

	function buildTree(entries: FileEntry[], query: string): TreeNodeData[] {
		const filtered = query
			? entries.filter((e) => e.path.toLowerCase().includes(query.toLowerCase()))
			: entries;

		const nodeMap = new Map<string, TreeNodeData>();
		const root: TreeNodeData[] = [];

		const sorted = [...filtered].sort((a, b) => {
			if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
			return a.path.localeCompare(b.path);
		});

		for (const entry of sorted) {
			const node: TreeNodeData = { entry, children: [] };
			nodeMap.set(entry.path, node);

			const parts = entry.path.split('/');
			if (parts.length === 1) {
				root.push(node);
			} else {
				const parentPath = parts.slice(0, -1).join('/');
				const parent = nodeMap.get(parentPath);
				if (parent) {
					parent.children.push(node);
				} else {
					root.push(node);
				}
			}
		}

		return root;
	}

	function toggleDir(path: string) {
		if (expandedDirs.has(path)) {
			expandedDirs.delete(path);
		} else {
			expandedDirs.add(path);
		}
		expandedDirs = new Set(expandedDirs);
	}

	async function handleSelect(path: string) {
		selectedFile.set(path);
		const wsId = $currentWorkspace?.id;
		if (!wsId) return;
		try {
			const content = await invoke<string>('read_file', { workspaceId: wsId, path });
			fileContent.set(content);
		} catch (e) {
			fileContent.set(`Error reading file: ${e}`);
		}
	}

	$: tree = buildTree($fileTree, searchQuery);
</script>

<div class="file-tree">
	<div class="search-wrap">
		<input
			class="search"
			type="text"
			placeholder="Filter files…"
			bind:value={searchQuery}
		/>
	</div>

	{#if $currentWorkspace}
		<div class="workspace-name">
			<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="13" height="13">
				<path d="M19.5 21a3 3 0 0 0 3-3v-4.5a3 3 0 0 0-3-3h-15a3 3 0 0 0-3 3V18a3 3 0 0 0 3 3h15zM1.5 10.146V6a3 3 0 0 1 3-3h5.379a2.25 2.25 0 0 1 1.59.659l2.122 2.121c.14.141.331.22.53.22H19.5a3 3 0 0 1 3 3v1.146A4.483 4.483 0 0 0 19.5 12h-15a4.483 4.483 0 0 0-3 1.146z" />
			</svg>
			{$currentWorkspace.name}
		</div>
	{/if}

	<div class="tree-body">
		{#if tree.length === 0}
			<div class="empty">No files found</div>
		{:else}
			{#each tree as node}
				<TreeNode
					{node}
					depth={0}
					{expandedDirs}
					selectedFile={$selectedFile}
					on:toggle={(e) => toggleDir(e.detail)}
					on:select={(e) => handleSelect(e.detail)}
				/>
			{/each}
		{/if}
	</div>
</div>

<style>
	.file-tree {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.search-wrap {
		padding: 8px;
		border-bottom: 1px solid #1e293b;
	}

	.search {
		width: 100%;
		box-sizing: border-box;
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 6px;
		color: #e2e8f0;
		padding: 6px 10px;
		font-size: 12px;
		outline: none;
	}

	.search::placeholder {
		color: #64748b;
	}

	.search:focus {
		border-color: #3b82f6;
	}

	.workspace-name {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		font-size: 11px;
		font-weight: 600;
		color: #64748b;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		border-bottom: 1px solid #1e293b;
	}

	.tree-body {
		flex: 1;
		overflow-y: auto;
		padding: 4px 0;
	}

	.empty {
		padding: 16px;
		font-size: 12px;
		color: #475569;
		text-align: center;
	}
</style>
