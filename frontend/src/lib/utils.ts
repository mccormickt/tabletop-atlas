// Simple className utility without external dependencies
export function cn(...inputs: (string | undefined | null | boolean)[]): string {
	return inputs.filter(Boolean).join(' ');
}

// API client base URL helper
export const API_BASE_URL = 'http://localhost:8080';

// Format date utility
export function formatDate(date: Date): string {
	return new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: 'short',
		day: 'numeric'
	}).format(date);
}

// Format date and time utility
export function formatDateTime(date: Date): string {
	return new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date);
}
