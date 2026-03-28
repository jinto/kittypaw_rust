<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { getSettings, saveApiKey } from '$lib/tauri';
	import PermissionSettings from './PermissionSettings.svelte';

	const dispatch = createEventDispatcher<{ close: void }>();

	export let activeWorkspaceId: string = '';

	let apiKey = '';
	let saved = false;
	let saving = false;
	let activeTab: 'general' | 'permissions' = 'general';

	onMount(async () => {
		try {
			apiKey = await getSettings();
		} catch (e) {
			console.error('Failed to load settings:', e);
		}
	});

	async function handleSave() {
		saving = true;
		try {
			await saveApiKey(apiKey);
			saved = true;
			setTimeout(() => (saved = false), 2000);
		} catch (e) {
			console.error('Failed to save API key:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="settings-overlay" tabindex="-1" on:click|self={() => dispatch('close')} on:keydown={(e) => { if (e.key === 'Escape') dispatch('close'); }} role="dialog" aria-modal="true" aria-label="Settings">
	<div class="settings-panel">
		<div class="header">
			<h2>Settings</h2>
			<button class="close-btn" on:click={() => dispatch('close')} aria-label="Close settings">✕</button>
		</div>

		<div class="tab-nav">
			<button class:active={activeTab === 'general'} on:click={() => (activeTab = 'general')}>General</button>
			<button class:active={activeTab === 'permissions'} on:click={() => (activeTab = 'permissions')}>Permissions</button>
		</div>

		{#if activeTab === 'general'}
			<div class="section">
				<label for="api-key">Anthropic API Key</label>
				<p class="hint">Your API key is stored locally and never sent anywhere except Anthropic's servers.</p>
				<input
					id="api-key"
					type="password"
					bind:value={apiKey}
					placeholder="sk-ant-..."
					autocomplete="off"
					spellcheck="false"
				/>
			</div>

			<div class="actions">
				<button class="save-btn" on:click={handleSave} disabled={saving}>
					{#if saving}
						Saving…
					{:else if saved}
						Saved ✓
					{:else}
						Save
					{/if}
				</button>
			</div>
		{:else if activeTab === 'permissions'}
			<PermissionSettings workspaceId={activeWorkspaceId} />
		{/if}
	</div>
</div>

<style>
	.settings-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.settings-panel {
		background: #fff;
		border-radius: 16px;
		padding: 28px;
		width: 520px;
		max-width: 94vw;
		max-height: 90vh;
		overflow-y: auto;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
	}

	.tab-nav {
		display: flex;
		gap: 4px;
		margin-bottom: 20px;
		border-bottom: 1px solid #e2e8f0;
		padding-bottom: 0;
	}

	.tab-nav button {
		background: none;
		border: none;
		padding: 7px 14px;
		font-size: 13px;
		color: #64748b;
		cursor: pointer;
		border-radius: 6px 6px 0 0;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		transition: color 0.15s;
	}

	.tab-nav button:hover {
		color: #1e293b;
	}

	.tab-nav button.active {
		color: #2563eb;
		border-bottom-color: #2563eb;
		font-weight: 600;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 24px;
	}

	h2 {
		font-size: 18px;
		font-weight: 600;
		color: #1e293b;
		margin: 0;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 18px;
		color: #94a3b8;
		cursor: pointer;
		padding: 4px;
		line-height: 1;
		border-radius: 4px;
	}

	.close-btn:hover {
		color: #1e293b;
		background: #f1f5f9;
	}

	.section {
		margin-bottom: 20px;
	}

	label {
		display: block;
		font-size: 13px;
		font-weight: 600;
		color: #374151;
		margin-bottom: 6px;
	}

	.hint {
		font-size: 12px;
		color: #6b7280;
		margin-bottom: 8px;
	}

	input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		font-family: monospace;
		outline: none;
		box-sizing: border-box;
		transition: border-color 0.15s;
	}

	input:focus {
		border-color: #2563eb;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
	}

	.save-btn {
		padding: 10px 24px;
		background: #2563eb;
		color: #fff;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
		min-width: 80px;
	}

	.save-btn:hover:not(:disabled) {
		background: #1d4ed8;
	}

	.save-btn:disabled {
		background: #93c5fd;
		cursor: not-allowed;
	}
</style>
