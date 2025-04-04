<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
	import type { KeyInfo, SigningOptions } from '$lib/types';
	import { FileText, Signature, Loader2 } from 'lucide-svelte';

	let { onBack } = $props();

	let keys = $state<KeyInfo[]>([]);
	let selectedKeyId = $state<string | null>(null);
	let documentPath = $state<string | null>(null);
	let password = $state('');
	let loadingKeys = $state(true);
	let signing = $state(false);
	let error = $state<string | null>(null);
	let successMessage = $state<string | null>(null);

	// This effect runs whenever 'keys' changes and 'loadingKeys' becomes false.
	$effect(() => {
		if (!loadingKeys && keys.length > 0 && selectedKeyId === null) {
			console.log('Effect: Auto-selecting first key');
			selectedKeyId = keys[0].keyId;
		}
	});

	// Fetch keys when the component mounts
	async function loadKeys() {
		loadingKeys = true;
		error = null;
		try {
			keys = await invoke<KeyInfo[]>('list_keys');
			// Automatically select the first key if none is selected yet
			if (keys.length > 0 && !selectedKeyId) {
				selectedKeyId = keys[0].keyId;
			}
		} catch (e) {
			console.error('Failed to load keys:', e);
			error = `Failed to load keys: ${e instanceof Error ? e.message : String(e)}`;
			keys = []; // Ensure keys array is empty on error
		} finally {
			loadingKeys = false;
		}
	}

	// Select the document to sign
	async function selectDocument() {
		error = null;
		successMessage = null; // Clear messages
		try {
			const selected = await openDialog({ multiple: false, title: 'Select Document to Sign' });
			documentPath = selected && typeof selected === 'string' ? selected : null;
		} catch (e) {
			error = `Failed select doc: ${e instanceof Error ? e.message : String(e)}`;
		}
	}

	// Perform the signing operation
	async function signDocument() {
		if (!documentPath || !selectedKeyId) {
			error = 'Please select a document and a key.';
			return;
		}
		if (!password) {
			error = 'Please enter the key password.';
			return;
		}

		signing = true;
		error = null;
		successMessage = null;

		try {
			const defaultSigPath = `${documentPath}.sig`;
			const outputPath = await saveDialog({
				title: 'Save Signature File',
				defaultPath: defaultSigPath,
				filters: [{ name: 'Signature', extensions: ['sig'] }]
			});

			if (!outputPath) {
				signing = false;
				return;
			} // User cancelled

			const options: SigningOptions = { format: 'detached' };

			await invoke<void>('sign_document', {
				documentPath: documentPath,
				keyId: selectedKeyId,
				password: password,
				outputPath: outputPath,
				options: options
			});
			successMessage = `Document signed successfully! Signature saved to: ${outputPath}`;
			password = ''; // Clear password
			// Optionally clear document path too? Depends on desired UX
			// documentPath = null;
		} catch (e) {
			/* ... error handling ... */ error = `Signing failed: ${e instanceof Error ? e.message : String(e)}`;
		} finally {
			signing = false;
		}
	}

	onMount(loadKeys);
</script>

<div class="sign-view-container">
	<form onsubmitcapture={signDocument}>
		<div class="form-group">
			<label for="document" class="form-label">Document to Sign:</label>
			<div class="file-input-group">
				<button
					type="button"
					class="button"
					onclick={selectDocument}
					disabled={signing || loadingKeys}
				>
					<FileText size={18} class="icon" /> Select Document
				</button>
				{#if documentPath}
					<span class="filepath">{documentPath}</span>
				{/if}
			</div>
		</div>

		<div class="form-group">
			<label for="key" class="form-label">Signing Key:</label>
			{#if loadingKeys}
				<p class="info">Loading keys...</p>
			{:else if keys.length > 0}
				<select
					id="key"
					class="form-control"
					bind:value={selectedKeyId}
					required
					disabled={signing}
				>
					{#each keys as key (key.keyId)}
						<option value={key.keyId}>{key.name} ({key.algorithm})</option>
					{/each}
				</select>
			{:else}
				<p>
					No signing keys available. Please <button
						type="button"
						class="button-link"
						onclick={onBack}>generate a key</button
					> first.
				</p>
			{/if}
		</div>

		<div class="form-group">
			<label for="password" class="form-label">Key Password:</label>
			<input
				id="password"
				type="password"
				class="form-control"
				bind:value={password}
				required
				disabled={signing || !selectedKeyId || keys.length === 0}
			/>
		</div>

		{#if error}
			<p class="alert alert-danger">{error}</p>
		{/if}
		{#if successMessage}
			<p class="alert alert-success">{successMessage}</p>
		{/if}

		<div class="form-actions">
			<button
				type="submit"
				class="button button-primary"
				disabled={signing || loadingKeys || !documentPath || !selectedKeyId || keys.length === 0}
			>
				{#if signing}
					<Loader2 size={18} class="icon" /> Signing...
				{:else}
					<Signature size={18} class="icon" /> Sign Document{/if}
			</button>
			<!-- Optional Back Button -->
			<!-- <button type="button" class="button" on:click={() => dispatch('back')} disabled={signing}>Back</button> -->
		</div>
	</form>
</div>

<style>
	.file-input-group {
		display: flex;
		align-items: center;
		flex-wrap: wrap; /* Allow wrapping */
	}
	.file-input-group button {
		flex-shrink: 0; /* Prevent button from shrinking */
	}
	.filepath {
		flex-grow: 1; /* Allow filepath to take space */
		min-width: 200px; /* Ensure some minimum width */
	}

	.form-actions {
		margin-top: var(--spacing-lg);
		padding-top: var(--spacing-md);
		border-top: 1px solid var(--color-border);
		display: flex;
		gap: var(--spacing-sm);
	}
	/* Spinner animation */
	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
	/* Info text style */
	.info {
		color: var(--color-text-muted);
		font-style: italic;
	}

	/* Remove redundant global styles */
</style>
