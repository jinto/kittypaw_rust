import { writable } from 'svelte/store';

export type MessageRole = 'user' | 'assistant' | 'error';

export interface Message {
	id: string;
	role: MessageRole;
	content: string;
	timestamp: Date;
}

function createChatStore() {
	const { subscribe, update } = writable<Message[]>([]);

	return {
		subscribe,
		addMessage(role: MessageRole, content: string): string {
			const id = crypto.randomUUID();
			update((msgs) => [
				...msgs,
				{ id, role, content, timestamp: new Date() }
			]);
			return id;
		},
		appendToMessage(id: string, token: string) {
			update((msgs) =>
				msgs.map((m) => (m.id === id ? { ...m, content: m.content + token } : m))
			);
		},
		updateMessage(id: string, content: string) {
			update((msgs) => msgs.map((m) => (m.id === id ? { ...m, content } : m)));
		},
		clear() {
			update(() => []);
		}
	};
}

export const chatStore = createChatStore();
export const isStreaming = writable(false);
