<script lang="ts">
	import { createEventDispatcher, tick } from 'svelte';
	import { fileTree } from '$lib/stores/workspace';
	import type { FileEntry } from '$lib/stores/workspace';

	export let disabled = false;

	const dispatch = createEventDispatcher<{ send: string }>();

	let value = '';
	let textareaEl: HTMLTextAreaElement;

	// @mention autocomplete state
	let showDropdown = false;
	let dropdownFiles: FileEntry[] = [];
	let atIndex = -1;
	let dropdownSelectedIndex = 0;

	function getAtQuery(): string | null {
		const pos = textareaEl?.selectionStart ?? value.length;
		const before = value.slice(0, pos);
		const match = before.match(/@([\w./\-]*)$/);
		return match ? match[1] : null;
	}

	function updateDropdown() {
		const query = getAtQuery();
		if (query === null) {
			showDropdown = false;
			return;
		}
		const q = query.toLowerCase();
		dropdownFiles = $fileTree
			.filter((f) => !f.is_dir && f.path.toLowerCase().includes(q))
			.slice(0, 8);
		showDropdown = dropdownFiles.length > 0;
		dropdownSelectedIndex = 0;

		const pos = textareaEl?.selectionStart ?? value.length;
		const before = value.slice(0, pos);
		const matchResult = before.match(/@([\w./\-]*)$/);
		atIndex = matchResult ? pos - matchResult[0].length : -1;
	}

	function selectFile(file: FileEntry) {
		const pos = textareaEl?.selectionStart ?? value.length;
		const before = value.slice(0, pos);
		const after = value.slice(pos);
		const matchResult = before.match(/@([\w./\-]*)$/);
		if (matchResult) {
			const start = pos - matchResult[0].length;
			value = value.slice(0, start) + '@' + file.path + ' ' + after;
		}
		showDropdown = false;
		tick().then(() => textareaEl?.focus());
	}

	function handleKeydown(e: KeyboardEvent) {
		if (showDropdown) {
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				dropdownSelectedIndex = Math.min(dropdownSelectedIndex + 1, dropdownFiles.length - 1);
				return;
			}
			if (e.key === 'ArrowUp') {
				e.preventDefault();
				dropdownSelectedIndex = Math.max(dropdownSelectedIndex - 1, 0);
				return;
			}
			if (e.key === 'Enter' || e.key === 'Tab') {
				e.preventDefault();
				if (dropdownFiles[dropdownSelectedIndex]) {
					selectFile(dropdownFiles[dropdownSelectedIndex]);
				}
				return;
			}
			if (e.key === 'Escape') {
				showDropdown = false;
				return;
			}
		}

		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			submit();
		}
	}

	function handleInput() {
		updateDropdown();
	}

	function submit() {
		const trimmed = value.trim();
		if (!trimmed || disabled) return;
		dispatch('send', trimmed);
		value = '';
		showDropdown = false;
	}

	// Render value with @mentions styled — we show them in the text only,
	// actual styled chip rendering requires a contenteditable which is complex.
	// Instead we parse and show a preview below the textarea when @mentions exist.
	$: mentions = [...value.matchAll(/@([\w./\-]+)/g)].map((m) => m[1]);
</script>

<div class="input-wrap">
	{#if mentions.length > 0}
		<div class="mentions-bar">
			{#each mentions as m}
				<span class="chip">@{m}</span>
			{/each}
		</div>
	{/if}

	<div class="input-row">
		{#if showDropdown}
			<div class="at-dropdown">
				{#each dropdownFiles as file, i}
					<button
						class="dropdown-item"
						class:selected={i === dropdownSelectedIndex}
						on:mousedown|preventDefault={() => selectFile(file)}
					>
						<span class="file-icon">📄</span>
						<span class="file-path">{file.path}</span>
					</button>
				{/each}
			</div>
		{/if}

		<textarea
			bind:this={textareaEl}
			bind:value
			on:keydown={handleKeydown}
			on:input={handleInput}
			placeholder="Message KittyPaw… (@ to mention a file, Enter to send)"
			{disabled}
			rows={1}
			class:disabled
		></textarea>
		<button on:click={submit} {disabled} class:disabled aria-label="Send">
			<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
				<path d="M3.478 2.405a.75.75 0 0 0-.926.94l2.432 7.905H13.5a.75.75 0 0 1 0 1.5H4.984l-2.432 7.905a.75.75 0 0 0 .926.94 60.519 60.519 0 0 0 18.445-8.986.75.75 0 0 0 0-1.218A60.517 60.517 0 0 0 3.478 2.405Z" />
			</svg>
		</button>
	</div>
</div>

<style>
	.input-wrap {
		background: #fff;
		border-top: 1px solid #e2e8f0;
		position: relative;
	}

	.mentions-bar {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		padding: 6px 16px 0;
	}

	.chip {
		background: #dbeafe;
		color: #1d4ed8;
		font-size: 11px;
		font-weight: 500;
		padding: 2px 8px;
		border-radius: 12px;
		font-family: 'SF Mono', 'Fira Code', monospace;
	}

	.input-row {
		display: flex;
		align-items: flex-end;
		gap: 8px;
		padding: 12px 16px;
		position: relative;
	}

	.at-dropdown {
		position: absolute;
		bottom: calc(100% - 4px);
		left: 16px;
		right: 56px;
		background: #fff;
		border: 1px solid #e2e8f0;
		border-radius: 10px;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.12);
		overflow: hidden;
		z-index: 100;
		max-height: 220px;
		overflow-y: auto;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		border: none;
		background: transparent;
		color: #475569;
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s;
	}

	.dropdown-item:hover,
	.dropdown-item.selected {
		background: #f1f5f9;
		color: #1e293b;
	}

	.file-path {
		font-family: 'SF Mono', 'Fira Code', monospace;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	textarea {
		flex: 1;
		resize: none;
		border: 1px solid #cbd5e1;
		border-radius: 12px;
		padding: 10px 14px;
		font-size: 14px;
		font-family: inherit;
		line-height: 1.5;
		outline: none;
		transition: border-color 0.15s;
		max-height: 140px;
		overflow-y: auto;
		field-sizing: content;
	}

	textarea:focus {
		border-color: #2563eb;
	}

	textarea.disabled {
		background: #f8fafc;
		color: #94a3b8;
		cursor: not-allowed;
	}

	button {
		flex-shrink: 0;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: 50%;
		background: #2563eb;
		color: #fff;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.15s;
	}

	button:hover:not(.disabled) {
		background: #1d4ed8;
	}

	button.disabled {
		background: #94a3b8;
		cursor: not-allowed;
	}
</style>
