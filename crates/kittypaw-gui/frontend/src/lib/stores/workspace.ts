import { writable } from 'svelte/store';

export interface FileEntry {
	path: string;
	size: number;
	modified: string;
	is_dir: boolean;
}

export interface FileChange {
	id: string;
	path: string;
	change_type: 'Create' | 'Modify' | 'Delete';
	diff: string;
	new_content: string;
	status: 'Pending' | 'Approved' | 'Rejected' | 'Applied';
}

export interface Workspace {
	id: string;
	name: string;
	root_path: string;
	created_at: string;
}

export const currentWorkspace = writable<Workspace | null>(null);
export const fileTree = writable<FileEntry[]>([]);
export const selectedFile = writable<string | null>(null);
export const fileContent = writable<string>('');
export const pendingChanges = writable<FileChange[]>([]);
