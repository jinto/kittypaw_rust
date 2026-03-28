<script lang="ts">
	import { onMount } from 'svelte';
	import type {
		FilePermissionRule,
		GlobalPath,
		NetworkPermissionRule
	} from '$lib/stores/permission';
	import {
		deleteFileRule,
		deleteGlobalPath,
		deleteNetworkRule,
		getPermissionProfile,
		saveFileRule,
		saveGlobalPath,
		saveNetworkRule
	} from '$lib/tauri';

	export let workspaceId: string;

	let fileRules: FilePermissionRule[] = [];
	let networkRules: NetworkPermissionRule[] = [];
	let globalPaths: GlobalPath[] = [];

	// New file rule form
	let newFilePath = '';
	let newFileRead = true;
	let newFileWrite = false;
	let newFileDelete = false;
	let newFileException = false;

	// New network rule form
	let newNetworkDomain = '';
	let newNetworkGet = true;
	let newNetworkPost = false;
	let newNetworkPut = false;
	let newNetworkDelete = false;

	// New global path form
	let newGlobalPath = '';
	let newGlobalAccessType: 'read' | 'write' = 'read';

	let activeTab: 'file' | 'network' | 'global' = 'file';
	let error = '';

	onMount(async () => {
		await loadProfile();
	});

	async function loadProfile() {
		if (!workspaceId) return;
		try {
			const profile = await getPermissionProfile(workspaceId);
			fileRules = profile.file_rules;
			networkRules = profile.network_rules;
			globalPaths = profile.global_paths;
		} catch (e) {
			error = `Failed to load permission profile: ${e}`;
		}
	}

	async function addFileRule() {
		if (!newFilePath.trim()) return;
		try {
			const id = await saveFileRule(
				workspaceId,
				newFilePath.trim(),
				newFileException,
				newFileRead,
				newFileWrite,
				newFileDelete
			);
			fileRules = [
				...fileRules,
				{
					id,
					workspace_id: workspaceId,
					path_pattern: newFilePath.trim(),
					is_exception: newFileException,
					can_read: newFileRead,
					can_write: newFileWrite,
					can_delete: newFileDelete
				}
			];
			newFilePath = '';
			newFileRead = true;
			newFileWrite = false;
			newFileDelete = false;
			newFileException = false;
		} catch (e) {
			error = `Failed to save file rule: ${e}`;
		}
	}

	async function removeFileRule(id: string) {
		try {
			await deleteFileRule(id);
			fileRules = fileRules.filter((r) => r.id !== id);
		} catch (e) {
			error = `Failed to delete file rule: ${e}`;
		}
	}

	async function addNetworkRule() {
		if (!newNetworkDomain.trim()) return;
		const methods: string[] = [];
		if (newNetworkGet) methods.push('GET');
		if (newNetworkPost) methods.push('POST');
		if (newNetworkPut) methods.push('PUT');
		if (newNetworkDelete) methods.push('DELETE');
		if (methods.length === 0) return;
		try {
			const id = await saveNetworkRule(workspaceId, newNetworkDomain.trim(), methods);
			networkRules = [
				...networkRules,
				{
					id,
					workspace_id: workspaceId,
					domain_pattern: newNetworkDomain.trim(),
					allowed_methods: methods
				}
			];
			newNetworkDomain = '';
			newNetworkGet = true;
			newNetworkPost = false;
			newNetworkPut = false;
			newNetworkDelete = false;
		} catch (e) {
			error = `Failed to save network rule: ${e}`;
		}
	}

	async function removeNetworkRule(id: string) {
		try {
			await deleteNetworkRule(id);
			networkRules = networkRules.filter((r) => r.id !== id);
		} catch (e) {
			error = `Failed to delete network rule: ${e}`;
		}
	}

	async function addGlobalPath() {
		if (!newGlobalPath.trim()) return;
		try {
			const id = await saveGlobalPath(newGlobalPath.trim(), newGlobalAccessType);
			globalPaths = [
				...globalPaths,
				{ id, path: newGlobalPath.trim(), access_type: newGlobalAccessType }
			];
			newGlobalPath = '';
			newGlobalAccessType = 'read';
		} catch (e) {
			error = `Failed to save global path: ${e}`;
		}
	}

	async function removeGlobalPath(id: string) {
		try {
			await deleteGlobalPath(id);
			globalPaths = globalPaths.filter((p) => p.id !== id);
		} catch (e) {
			error = `Failed to delete global path: ${e}`;
		}
	}
</script>

<div class="permission-settings">
	{#if error}
		<div class="error-banner">{error}</div>
	{/if}

	{#if !workspaceId}
		<p class="hint">Open a workspace to manage its permission rules.</p>
	{:else}
		<div class="tab-bar">
			<button class:active={activeTab === 'file'} on:click={() => (activeTab = 'file')}>
				File Rules
			</button>
			<button class:active={activeTab === 'network'} on:click={() => (activeTab = 'network')}>
				Network Rules
			</button>
			<button class:active={activeTab === 'global'} on:click={() => (activeTab = 'global')}>
				Global Paths
			</button>
		</div>

		{#if activeTab === 'file'}
			<div class="section">
				<p class="section-hint">
					Control which file paths the agent can read, write, or delete. Exception rules override
					normal rules.
				</p>

				{#if fileRules.length > 0}
					<table class="rules-table">
						<thead>
							<tr>
								<th>Pattern</th>
								<th>Read</th>
								<th>Write</th>
								<th>Delete</th>
								<th>Exception</th>
								<th></th>
							</tr>
						</thead>
						<tbody>
							{#each fileRules as rule (rule.id)}
								<tr>
									<td class="mono">{rule.path_pattern}</td>
									<td class="center">{rule.can_read ? '✓' : '—'}</td>
									<td class="center">{rule.can_write ? '✓' : '—'}</td>
									<td class="center">{rule.can_delete ? '✓' : '—'}</td>
									<td class="center">{rule.is_exception ? '✓' : '—'}</td>
									<td>
										<button class="remove-btn" on:click={() => removeFileRule(rule.id)}>✕</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{:else}
					<p class="empty">No file rules configured.</p>
				{/if}

				<div class="add-form">
					<input
						class="path-input"
						type="text"
						placeholder="Path pattern (e.g. /src, *.env)"
						bind:value={newFilePath}
					/>
					<div class="checkbox-row">
						<label><input type="checkbox" bind:checked={newFileRead} /> Read</label>
						<label><input type="checkbox" bind:checked={newFileWrite} /> Write</label>
						<label><input type="checkbox" bind:checked={newFileDelete} /> Delete</label>
						<label><input type="checkbox" bind:checked={newFileException} /> Exception</label>
					</div>
					<button class="add-btn" on:click={addFileRule} disabled={!newFilePath.trim()}>
						Add Rule
					</button>
				</div>
			</div>

		{:else if activeTab === 'network'}
			<div class="section">
				<p class="section-hint">
					Control which domains the agent can contact and which HTTP methods are allowed.
				</p>

				{#if networkRules.length > 0}
					<table class="rules-table">
						<thead>
							<tr>
								<th>Domain Pattern</th>
								<th>Allowed Methods</th>
								<th></th>
							</tr>
						</thead>
						<tbody>
							{#each networkRules as rule (rule.id)}
								<tr>
									<td class="mono">{rule.domain_pattern}</td>
									<td>{rule.allowed_methods.join(', ')}</td>
									<td>
										<button class="remove-btn" on:click={() => removeNetworkRule(rule.id)}>✕</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{:else}
					<p class="empty">No network rules configured.</p>
				{/if}

				<div class="add-form">
					<input
						class="path-input"
						type="text"
						placeholder="Domain pattern (e.g. api.example.com, *.internal.com)"
						bind:value={newNetworkDomain}
					/>
					<div class="checkbox-row">
						<label><input type="checkbox" bind:checked={newNetworkGet} /> GET</label>
						<label><input type="checkbox" bind:checked={newNetworkPost} /> POST</label>
						<label><input type="checkbox" bind:checked={newNetworkPut} /> PUT</label>
						<label><input type="checkbox" bind:checked={newNetworkDelete} /> DELETE</label>
					</div>
					<button class="add-btn" on:click={addNetworkRule} disabled={!newNetworkDomain.trim()}>
						Add Rule
					</button>
				</div>
			</div>

		{:else if activeTab === 'global'}
			<div class="section">
				<p class="section-hint">
					Global paths apply across all workspaces (e.g. a shared read-only data directory).
				</p>

				{#if globalPaths.length > 0}
					<table class="rules-table">
						<thead>
							<tr>
								<th>Path</th>
								<th>Access</th>
								<th></th>
							</tr>
						</thead>
						<tbody>
							{#each globalPaths as gp (gp.id)}
								<tr>
									<td class="mono">{gp.path}</td>
									<td>{gp.access_type}</td>
									<td>
										<button class="remove-btn" on:click={() => removeGlobalPath(gp.id)}>✕</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{:else}
					<p class="empty">No global paths configured.</p>
				{/if}

				<div class="add-form">
					<input
						class="path-input"
						type="text"
						placeholder="Absolute path (e.g. /home/user/shared)"
						bind:value={newGlobalPath}
					/>
					<div class="checkbox-row">
						<label>
							<input type="radio" bind:group={newGlobalAccessType} value="read" /> Read
						</label>
						<label>
							<input type="radio" bind:group={newGlobalAccessType} value="write" /> Write
						</label>
					</div>
					<button class="add-btn" on:click={addGlobalPath} disabled={!newGlobalPath.trim()}>
						Add Path
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>

<style>
	.permission-settings {
		padding: 4px 0;
	}

	.error-banner {
		background: #fee2e2;
		color: #b91c1c;
		padding: 10px 14px;
		border-radius: 8px;
		font-size: 13px;
		margin-bottom: 16px;
	}

	.hint {
		font-size: 13px;
		color: #6b7280;
	}

	.tab-bar {
		display: flex;
		gap: 4px;
		margin-bottom: 20px;
		border-bottom: 1px solid #e2e8f0;
		padding-bottom: 0;
	}

	.tab-bar button {
		background: none;
		border: none;
		padding: 8px 14px;
		font-size: 13px;
		color: #64748b;
		cursor: pointer;
		border-radius: 6px 6px 0 0;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		transition: color 0.15s;
	}

	.tab-bar button:hover {
		color: #1e293b;
	}

	.tab-bar button.active {
		color: #2563eb;
		border-bottom-color: #2563eb;
		font-weight: 600;
	}

	.section-hint {
		font-size: 12px;
		color: #6b7280;
		margin-bottom: 14px;
	}

	.rules-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
		margin-bottom: 16px;
	}

	.rules-table th {
		text-align: left;
		font-size: 11px;
		font-weight: 600;
		color: #64748b;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding: 6px 8px;
		border-bottom: 1px solid #e2e8f0;
	}

	.rules-table td {
		padding: 8px;
		border-bottom: 1px solid #f1f5f9;
		color: #1e293b;
	}

	.mono {
		font-family: monospace;
		font-size: 12px;
	}

	.center {
		text-align: center;
	}

	.empty {
		font-size: 13px;
		color: #94a3b8;
		margin-bottom: 16px;
	}

	.add-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		background: #f8fafc;
		border-radius: 10px;
		border: 1px solid #e2e8f0;
	}

	.path-input {
		width: 100%;
		padding: 8px 12px;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 13px;
		font-family: monospace;
		outline: none;
		box-sizing: border-box;
		transition: border-color 0.15s;
	}

	.path-input:focus {
		border-color: #2563eb;
	}

	.checkbox-row {
		display: flex;
		gap: 16px;
		flex-wrap: wrap;
	}

	.checkbox-row label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: #374151;
		cursor: pointer;
		font-weight: normal;
	}

	.add-btn {
		align-self: flex-end;
		padding: 8px 20px;
		background: #2563eb;
		color: #fff;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
	}

	.add-btn:hover:not(:disabled) {
		background: #1d4ed8;
	}

	.add-btn:disabled {
		background: #93c5fd;
		cursor: not-allowed;
	}

	.remove-btn {
		background: none;
		border: none;
		color: #94a3b8;
		cursor: pointer;
		padding: 2px 6px;
		font-size: 13px;
		border-radius: 4px;
		transition: color 0.15s, background 0.15s;
	}

	.remove-btn:hover {
		color: #dc2626;
		background: #fee2e2;
	}
</style>
