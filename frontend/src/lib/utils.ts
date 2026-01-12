import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

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

// Similarity score utilities
export function formatSimilarityScore(score: number): string {
	return (score * 100).toFixed(1) + '%';
}

export function getSimilarityBadgeVariant(score: number): 'default' | 'secondary' | 'outline' {
	if (score >= 0.8) return 'default';
	if (score >= 0.6) return 'secondary';
	return 'outline';
}

export function getSimilarityColor(score: number): string {
	if (score >= 0.8) return 'text-green-600';
	if (score >= 0.6) return 'text-yellow-600';
	return 'text-gray-600';
}

// Text utilities
export function truncateText(text: string, maxLength: number): string {
	if (text.length <= maxLength) return text;
	return text.substring(0, maxLength) + '...';
}

// Game formatting utilities
export function formatPlayerCount(min?: number, max?: number): string {
	if (min === undefined && max === undefined) return 'Not specified';
	if (min === max || max === undefined) return `${min} players`;
	if (min === undefined) return `Up to ${max} players`;
	return `${min}-${max} players`;
}

export function formatPlayTime(minutes?: number): string {
	if (minutes === undefined) return 'Not specified';
	if (minutes < 60) return `${minutes} min`;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

export function formatComplexity(rating?: number): string {
	if (rating === undefined) return 'Not rated';
	return `${rating.toFixed(1)}/5`;
}

// Challenge status utilities
export type ChallengeStatusType = 'draft' | 'active' | 'completed' | 'archived';

export function getStatusColor(status: string): string {
	switch (status) {
		case 'active':
			return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
		case 'completed':
			return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
		case 'draft':
			return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
		case 'archived':
			return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
		default:
			return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
	}
}
