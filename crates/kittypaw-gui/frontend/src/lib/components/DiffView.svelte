<script lang="ts">
	import { pendingChanges } from '$lib/stores/workspace';
	import type { FileChange } from '$lib/stores/workspace';
	import { invoke } from '@tauri-apps/api/core';

	export let change: FileChange;

	let loading = false;

	function parseDiff(diff: string): Array<{ type: 'add' | 'remove' | 'context' | 'header'; text: string }> {
		return diff.split('\n').map((line) => {
			if (line.startsWith('+++') || line.startsWith('---') || line.startsWith('@@')) {
				return { type: 'header', text: line };
			} else if (line.startsWith('+')) {
				return { type: 'add', text: line };
			} else if (line.startsWith('-')) {
				return { type: 'remove', text: line };
			} else {
				return { type: 'context', text: line };
			}
		});
	}

	async function approve() {
		loading = true;
		try {
			await invoke('approve_change', { changeId: change.id });
			pendingChanges.update((changes) =>
				changes.map((c) => (c.id === change.id ? { ...c, status: 'Approved' } : c))
			);
		} catch (e) {
			console.error('approve_change error:', e);
		} finally {
			loading = false;
		}
	}

	async function reject() {
		loading = true;
		try {
			await invoke('reject_change', { changeId: change.id });
			pendingChanges.update((changes) =>
				changes.map((c) => (c.id === change.id ? { ...c, status: 'Rejected' } : c))
			);
		} catch (e) {
			console.error('reject_change error:', e);
		} finally {
			loading = false;
		}
	}

	$: lines = parseDiff(change.diff);

	const statusColors: Record<FileChange['status'], string> = {
		Pending: '#f59e0b',
		Approved: '#22c55e',
		Rejected: '#ef4444',
		Applied: '#3b82f6'
	};
</script>

<div class="diff-view">
	<div class="diff-header">
		<div class="file-info">
			<span class="change-type" data-type={change.change_type.toLowerCase()}>{change.change_type}</span>
			<span class="file-path">{change.path}</span>
		</div>
		<div class="status" style="color: {statusColors[change.status]}">
			{change.status}
		</div>
	</div>

	<div class="diff-body">
		<pre class="diff-code"><code>{#each lines as line}<span class="line {line.type}">{line.text}
</span>{/each}</code></pre>
	</div>

	{#if change.status === 'Pending'}
		<div class="diff-actions">
			<button class="btn reject" on:click={reject} disabled={loading}>
				<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
					<line x1="18" y1="6" x2="6" y2="18"></line>
					<line x1="6" y1="6" x2="18" y2="18"></line>
				</svg>
				Reject
			</button>
			<button class="btn approve" on:click={approve} disabled={loading}>
				<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
					<polyline points="20 6 9 17 4 12"></polyline>
				</svg>
				Approve
			</button>
		</div>
	{/if}
</div>

<style>
	.diff-view {
		display: flex;
		flex-direction: column;
		background: #fff;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		overflow: hidden;
		margin-bottom: 12px;
	}

	.diff-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		background: #f8fafc;
		border-bottom: 1px solid #e2e8f0;
		font-size: 13px;
	}

	.file-info {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.change-type {
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		padding: 2px 7px;
		border-radius: 4px;
	}

	.change-type[data-type='create'] {
		background: #dcfce7;
		color: #166534;
	}

	.change-type[data-type='modify'] {
		background: #dbeafe;
		color: #1e40af;
	}

	.change-type[data-type='delete'] {
		background: #fee2e2;
		color: #991b1b;
	}

	.file-path {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 12px;
		color: #475569;
	}

	.status {
		font-size: 12px;
		font-weight: 600;
	}

	.diff-body {
		overflow: auto;
		max-height: 320px;
	}

	.diff-code {
		margin: 0;
		font-family: 'SF Mono', 'Fira Code', 'Fira Mono', 'Roboto Mono', monospace;
		font-size: 12px;
		line-height: 1.5;
		tab-size: 4;
	}

	.line {
		display: block;
		padding: 0 12px;
		white-space: pre;
	}

	.line.add {
		background: #f0fdf4;
		color: #166534;
	}

	.line.remove {
		background: #fff1f2;
		color: #991b1b;
	}

	.line.context {
		color: #475569;
	}

	.line.header {
		background: #f1f5f9;
		color: #64748b;
		font-style: italic;
	}

	.diff-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
		padding: 10px 14px;
		border-top: 1px solid #e2e8f0;
	}

	.btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border: none;
		border-radius: 7px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
	}

	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn.approve {
		background: #22c55e;
		color: #fff;
	}

	.btn.approve:hover:not(:disabled) {
		background: #16a34a;
	}

	.btn.reject {
		background: #f1f5f9;
		color: #475569;
	}

	.btn.reject:hover:not(:disabled) {
		background: #fee2e2;
		color: #ef4444;
	}
</style>
