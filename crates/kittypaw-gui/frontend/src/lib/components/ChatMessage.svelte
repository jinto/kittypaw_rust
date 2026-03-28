<script lang="ts">
	import type { Message } from '$lib/stores/chat';

	export let message: Message;

	function formatTime(date: Date): string {
		return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}
</script>

<div class="message {message.role}">
	<div class="bubble">
		<div class="content">{message.content}</div>
		<div class="timestamp">{formatTime(message.timestamp)}</div>
	</div>
</div>

<style>
	.message {
		display: flex;
		margin-bottom: 12px;
	}

	.message.user {
		justify-content: flex-end;
	}

	.message.assistant,
	.message.error {
		justify-content: flex-start;
	}

	.bubble {
		max-width: 72%;
		padding: 10px 14px;
		border-radius: 18px;
		font-size: 14px;
		line-height: 1.5;
		word-break: break-word;
	}

	.user .bubble {
		background: #2563eb;
		color: #fff;
		border-bottom-right-radius: 4px;
	}

	.assistant .bubble {
		background: #f1f5f9;
		color: #1e293b;
		border-bottom-left-radius: 4px;
	}

	.error .bubble {
		background: #fee2e2;
		color: #b91c1c;
		border-bottom-left-radius: 4px;
	}

	.content {
		white-space: pre-wrap;
	}

	.timestamp {
		font-size: 11px;
		margin-top: 4px;
		opacity: 0.6;
		text-align: right;
	}
</style>
