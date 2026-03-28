import { writable } from 'svelte/store';

export interface FilePermissionRule {
	id: string;
	workspace_id: string;
	path_pattern: string;
	is_exception: boolean;
	can_read: boolean;
	can_write: boolean;
	can_delete: boolean;
}

export interface NetworkPermissionRule {
	id: string;
	workspace_id: string;
	domain_pattern: string;
	allowed_methods: string[];
}

export interface GlobalPath {
	id: string;
	path: string;
	access_type: 'read' | 'write';
}

export interface PermissionProfile {
	workspace_id: string;
	file_rules: FilePermissionRule[];
	network_rules: NetworkPermissionRule[];
	global_paths: GlobalPath[];
}

export interface PermissionRequest {
	request_id: string;
	resource_kind: 'file' | 'network';
	resource_path: string;
	action: string;
	workspace_id: string;
}

export const permissionProfile = writable<PermissionProfile | null>(null);
export const pendingPermissionRequest = writable<PermissionRequest | null>(null);
