<script lang="ts">
	import { api } from '$lib';
	import { createEventDispatcher } from 'svelte';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Progress } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import type { RulesInfoResponse, UploadResponse } from '$lib';

	// Props
	let {
		gameId,
		gameName = '',
		existingRulesInfo = null
	}: {
		gameId: number;
		gameName?: string;
		existingRulesInfo?: RulesInfoResponse | null;
	} = $props();

	// State
	let isDragging = $state(false);
	let isUploading = $state(false);
	let uploadProgress = $state(0);
	let selectedFile: File | null = $state(null);
	let error = $state<string | null>(null);
	let uploadResult: UploadResponse | null = $state(null);
	let fileInputRef: HTMLInputElement | null = $state(null);

	// Event dispatcher
	const dispatch = createEventDispatcher<{
		uploaded: UploadResponse;
		deleted: void;
		error: string;
	}>();

	// File validation
	function validateFile(file: File): string | null {
		// Check file type
		if (file.type !== 'application/pdf') {
			return 'Please select a PDF file';
		}

		// Check file size (10MB limit)
		const maxSize = 10 * 1024 * 1024; // 10MB
		if (file.size > maxSize) {
			return 'File size must be less than 10MB';
		}

		return null;
	}

	// Handle file selection
	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const files = target.files;
		if (files && files.length > 0) {
			const file = files[0];
			const validationError = validateFile(file);
			if (validationError) {
				error = validationError;
				selectedFile = null;
			} else {
				selectedFile = file;
				error = null;
			}
		}
	}

	// Handle drag and drop
	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		isDragging = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		isDragging = false;
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragging = false;

		const files = event.dataTransfer?.files;
		if (files && files.length > 0) {
			const file = files[0];
			const validationError = validateFile(file);
			if (validationError) {
				error = validationError;
				selectedFile = null;
			} else {
				selectedFile = file;
				error = null;
			}
		}
	}

	// Handle button click to open file dialog
	function openFileDialog() {
		fileInputRef?.click();
	}

	// Upload file
	async function uploadFile() {
		if (!selectedFile) return;

		isUploading = true;
		uploadProgress = 0;
		error = null;
		uploadResult = null;

		try {
			// Simulate progress for user feedback
			const progressInterval = setInterval(() => {
				if (uploadProgress < 90) {
					uploadProgress += Math.random() * 10;
				}
			}, 200);

			const result = await api.methods.uploadRulesPdf({
				path: { id: gameId },
				body: selectedFile
			});

			clearInterval(progressInterval);
			uploadProgress = 100;

			if (result.type === 'success') {
				uploadResult = result.data;
				selectedFile = null;
				dispatch('uploaded', result.data);
			} else if (result.type === 'error') {
				error = result.data.message || 'Upload failed';
				dispatch('error', error);
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Upload failed';
				dispatch('error', error);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			dispatch('error', error);
		} finally {
			isUploading = false;
			uploadProgress = 0;
		}
	}

	// Delete existing rules
	async function deleteRules() {
		if (
			!confirm('Are you sure you want to delete the uploaded rules? This action cannot be undone.')
		) {
			return;
		}

		try {
			const result = await api.methods.deleteRules({
				path: { id: gameId }
			});

			if (result.type === 'success') {
				dispatch('deleted');
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to delete rules';
				dispatch('error', error);
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to delete rules';
				dispatch('error', error);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			dispatch('error', error);
		}
	}

	// Format file size
	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 Bytes';
		const k = 1024;
		const sizes = ['Bytes', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	// Format date
	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString();
	}
</script>

<Card class="w-full">
	<CardHeader>
		<CardTitle class="flex items-center space-x-2">
			<svg class="h-5 w-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
				></path>
			</svg>
			<span>PDF Rules Upload</span>
		</CardTitle>
		<CardDescription>
			Upload a PDF file containing the game rules for {gameName || `Game #${gameId}`}
		</CardDescription>
	</CardHeader>
	<CardContent class="space-y-4">
		<!-- Error Display -->
		{#if error}
			<div class="rounded-md border border-red-200 bg-red-50 p-3">
				<div class="flex">
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">Upload Error</h3>
						<div class="mt-2 text-sm text-red-700">
							<p>{error}</p>
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Success Display -->
		{#if uploadResult}
			<div class="rounded-md border border-green-200 bg-green-50 p-3">
				<div class="flex">
					<div class="ml-3">
						<h3 class="text-sm font-medium text-green-800">Upload Successful!</h3>
						<div class="mt-2 text-sm text-green-700">
							<p>{uploadResult.message}</p>
							{#if uploadResult.chunksProcessed}
								<p class="mt-1">
									Processed {uploadResult.chunksProcessed} text chunks
									{#if uploadResult.textLength}
										from {uploadResult.textLength.toLocaleString()} characters
									{/if}
								</p>
							{/if}
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Existing Rules Info -->
		{#if existingRulesInfo?.hasRulesPdf}
			<div class="rounded-md border border-blue-200 bg-blue-50 p-4">
				<div class="flex items-start justify-between">
					<div class="flex">
						<svg
							class="mt-0.5 h-5 w-5 text-blue-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
							></path>
						</svg>
						<div class="ml-3">
							<h3 class="text-sm font-medium text-blue-800">Rules Already Uploaded</h3>
							<div class="mt-1 text-sm text-blue-700">
								<p>
									{existingRulesInfo.chunkCount} text chunks processed
									{#if existingRulesInfo.textLength}
										({existingRulesInfo.textLength.toLocaleString()} characters)
									{/if}
								</p>
								{#if existingRulesInfo.lastProcessed}
									<p class="mt-1">Last processed: {formatDate(existingRulesInfo.lastProcessed)}</p>
								{/if}
							</div>
						</div>
					</div>
					<Button variant="destructive" size="sm" onclick={deleteRules}>Replace</Button>
				</div>
			</div>
		{:else}
			<!-- Upload Area -->
			<div
				class="relative rounded-lg border-2 border-dashed p-6 transition-colors
							{isDragging
					? 'border-blue-400 bg-blue-50'
					: selectedFile
						? 'border-green-400 bg-green-50'
						: 'border-gray-300 hover:border-gray-400'}"
				role="button"
				tabindex="0"
				ondragover={handleDragOver}
				ondragleave={handleDragLeave}
				ondrop={handleDrop}
			>
				{#if selectedFile}
					<!-- Selected File Display -->
					<div class="text-center">
						<svg
							class="mx-auto h-12 w-12 text-green-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
							></path>
						</svg>
						<div class="mt-4">
							<p class="text-sm font-medium text-gray-900">{selectedFile.name}</p>
							<p class="text-sm text-gray-500">{formatFileSize(selectedFile.size)}</p>
						</div>
						<div class="mt-4 flex justify-center space-x-3">
							<Button onclick={uploadFile} disabled={isUploading}>
								{#if isUploading}
									Uploading...
								{:else}
									Upload PDF
								{/if}
							</Button>
							<Button
								variant="outline"
								onclick={() => (selectedFile = null)}
								disabled={isUploading}
							>
								Cancel
							</Button>
						</div>
					</div>
				{:else}
					<!-- Drop Zone -->
					<div class="text-center">
						<svg
							class="mx-auto h-12 w-12 text-gray-400"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
							></path>
						</svg>
						<div class="mt-4">
							<p class="text-lg font-medium text-gray-900">Drop your PDF file here</p>
							<p class="text-sm text-gray-500">or click to browse files</p>
						</div>
						<div class="mt-6">
							<Button onclick={openFileDialog}>Select PDF File</Button>
							<input
								bind:this={fileInputRef}
								type="file"
								class="sr-only"
								accept=".pdf,application/pdf"
								onchange={handleFileSelect}
							/>
						</div>
						<div class="mt-4">
							<div class="flex items-center space-x-4 text-sm text-gray-500">
								<Badge variant="secondary">PDF Only</Badge>
								<Badge variant="secondary">Max 10MB</Badge>
							</div>
						</div>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Upload Progress -->
		{#if isUploading}
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<span class="text-sm font-medium text-gray-700">Uploading and processing PDF...</span>
					<span class="text-sm text-gray-500">{Math.round(uploadProgress)}%</span>
				</div>
				<Progress value={uploadProgress} class="h-2" />
				<p class="text-xs text-gray-500">
					This may take a moment as we extract text and generate embeddings
				</p>
			</div>
		{/if}

		<!-- Help Text -->
		<div class="text-sm text-gray-600">
			<p class="font-medium">Tips for best results:</p>
			<ul class="mt-1 list-inside list-disc space-y-1">
				<li>Upload official rulebooks for accurate information</li>
				<li>Ensure the PDF contains searchable text (not just images)</li>
				<li>Smaller files upload and process faster</li>
			</ul>
		</div>
	</CardContent>
</Card>
