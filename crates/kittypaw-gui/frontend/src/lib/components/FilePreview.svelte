<script lang="ts">
	import { selectedFile, fileContent } from '$lib/stores/workspace';
</script>

{#if $selectedFile}
	<div class="preview">
		<div class="header">
			<span class="icon">📄</span>
			<span class="path">{$selectedFile}</span>
		</div>
		<div class="code-wrap">
			<pre class="code"><code>{$fileContent}</code></pre>
		</div>
	</div>
{:else}
	<div class="empty">
		<div class="empty-icon">📄</div>
		<p>Select a file to preview</p>
	</div>
{/if}

<style>
	.preview {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #fff;
		overflow: hidden;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 16px;
		background: #f8fafc;
		border-bottom: 1px solid #e2e8f0;
		font-size: 13px;
		color: #475569;
		font-weight: 500;
	}

	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: #1e293b;
	}

	.code-wrap {
		flex: 1;
		overflow: auto;
		padding: 16px;
	}

	.code {
		margin: 0;
		font-family: 'SF Mono', 'Fira Code', 'Fira Mono', 'Roboto Mono', monospace;
		font-size: 13px;
		line-height: 1.6;
		color: #1e293b;
		white-space: pre;
		tab-size: 4;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: #94a3b8;
	}

	.empty-icon {
		font-size: 40px;
		margin-bottom: 12px;
	}

	.empty p {
		font-size: 14px;
		margin: 0;
	}
</style>
