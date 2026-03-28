<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { currentWorkspace } from '$lib/stores/workspace';

	export let workspaceId: string = '';

	interface SearchResult {
		path: string;
		score: number;
		snippet: string;
	}

	interface SemanticResult {
		path: string;
		rank: number;
		reason: string;
	}

	type AnyResult = { path: string; detail: string };

	let query = '';
	let mode: 'keyword' | 'semantic' = 'keyword';
	let results: AnyResult[] = [];
	let loading = false;
	let error = '';

	export let onFileSelect: (path: string) => void = () => {};

	async function runSearch() {
		if (!query.trim()) {
			results = [];
			return;
		}

		const wsId = workspaceId || ($currentWorkspace?.id ?? '');
		if (!wsId) {
			error = 'No workspace open.';
			return;
		}

		loading = true;
		error = '';
		results = [];

		try {
			if (mode === 'keyword') {
				const raw: SearchResult[] = await invoke('search_files', {
					query,
					workspaceId: wsId,
				});
				results = raw.map((r) => ({
					path: r.path,
					detail: r.snippet || `score: ${r.score.toFixed(3)}`,
				}));
			} else {
				const raw: SemanticResult[] = await invoke('semantic_search', {
					query,
					workspaceId: wsId,
				});
				results = raw.map((r) => ({
					path: r.path,
					detail: r.reason || `rank: ${r.rank}`,
				}));
			}
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') runSearch();
	}
</script>

<div class="search-bar">
	<div class="mode-toggle">
		<button
			class="mode-btn"
			class:active={mode === 'keyword'}
			on:click={() => (mode = 'keyword')}
		>
			Keyword
		</button>
		<button
			class="mode-btn"
			class:active={mode === 'semantic'}
			on:click={() => (mode = 'semantic')}
		>
			Semantic
		</button>
	</div>

	<div class="input-row">
		<input
			type="text"
			class="search-input"
			placeholder={mode === 'keyword' ? 'Search files...' : 'Describe what you need...'}
			bind:value={query}
			on:keydown={handleKeydown}
		/>
		<button class="search-btn" on:click={runSearch} disabled={loading}>
			{#if loading}
				<span class="spinner"></span>
			{:else}
				<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
					<circle cx="11" cy="11" r="8"></circle>
					<line x1="21" y1="21" x2="16.65" y2="16.65"></line>
				</svg>
			{/if}
		</button>
	</div>

	{#if error}
		<div class="search-error">{error}</div>
	{/if}

	{#if results.length > 0}
		<ul class="results-list">
			{#each results as result}
				<li>
					<button class="result-item" on:click={() => onFileSelect(result.path)}>
						<span class="result-path">{result.path}</span>
						{#if result.detail}
							<span class="result-detail">{result.detail}</span>
						{/if}
					</button>
				</li>
			{/each}
		</ul>
	{:else if !loading && query.trim()}
		<div class="no-results">No results found.</div>
	{/if}
</div>

<style>
	.search-bar {
		padding: 8px 12px;
		border-top: 1px solid #1e293b;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.mode-toggle {
		display: flex;
		gap: 4px;
	}

	.mode-btn {
		flex: 1;
		padding: 4px 8px;
		border-radius: 6px;
		border: 1px solid #334155;
		background: transparent;
		color: #64748b;
		font-size: 11px;
		cursor: pointer;
		transition: background 0.15s, color 0.15s;
	}

	.mode-btn:hover {
		background: #1e293b;
		color: #94a3b8;
	}

	.mode-btn.active {
		background: #1e3a5f;
		color: #93c5fd;
		border-color: #3b82f6;
	}

	.input-row {
		display: flex;
		gap: 4px;
	}

	.search-input {
		flex: 1;
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 6px;
		padding: 6px 8px;
		color: #e2e8f0;
		font-size: 12px;
		outline: none;
	}

	.search-input:focus {
		border-color: #3b82f6;
	}

	.search-input::placeholder {
		color: #475569;
	}

	.search-btn {
		background: #1e3a5f;
		border: 1px solid #3b82f6;
		border-radius: 6px;
		padding: 6px 8px;
		color: #93c5fd;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.15s;
	}

	.search-btn:hover {
		background: #2563eb;
		color: #fff;
	}

	.search-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.spinner {
		width: 14px;
		height: 14px;
		border: 2px solid #93c5fd;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
		display: inline-block;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.search-error {
		font-size: 11px;
		color: #f87171;
		padding: 2px 0;
	}

	.results-list {
		list-style: none;
		margin: 0;
		padding: 0;
		max-height: 200px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.results-list li {
		padding: 0;
	}

	.result-item {
		width: 100%;
		padding: 6px 8px;
		border-radius: 6px;
		cursor: pointer;
		background: #0f172a;
		border: 1px solid #1e293b;
		transition: background 0.1s;
		text-align: left;
		display: flex;
		flex-direction: column;
	}

	.result-item:hover {
		background: #1e293b;
		border-color: #334155;
	}

	.result-path {
		display: block;
		font-size: 11px;
		color: #93c5fd;
		font-family: monospace;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.result-detail {
		display: block;
		font-size: 10px;
		color: #64748b;
		margin-top: 2px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.no-results {
		font-size: 11px;
		color: #475569;
		text-align: center;
		padding: 8px 0;
	}
</style>
