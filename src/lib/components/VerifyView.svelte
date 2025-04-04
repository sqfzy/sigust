<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import type { KeyInfo, VerificationResult } from '$lib/types';
	import { FileText, KeyRound, Search, Loader2, CheckCircle2, XCircle } from 'lucide-svelte';

	let { onBack } = $props();

	let keys = $state<KeyInfo[]>([]);
	let selectedKeyId = $state<string | null>(null);
	let documentPath = $state<string | null>(null);
	let signaturePath = $state<string | null>(null);
	let loadingKeys = $state(true);
	let verifying = $state(false);
	let error = $state<string | null>(null); // Process errors
	let verificationResult = $state<VerificationResult | null>(null); // Result object

	// --- >>> Use $effect to automatically select first key <<< ---
	$effect(() => {
		if (!loadingKeys && keys.length > 0 && selectedKeyId === null) {
			console.log('Effect: Auto-selecting first key for verification');
			selectedKeyId = keys[0].keyId;
		}
		// Also reset selection if keys become empty
		if (!loadingKeys && keys.length === 0) {
			selectedKeyId = null;
		}
	});

	// Fetch keys when component mounts
	async function loadKeys() {
		loadingKeys = true;
		error = null;
		try {
			keys = await invoke<KeyInfo[]>('list_keys');
			if (keys.length > 0 && !selectedKeyId) {
				selectedKeyId = keys[0].keyId;
			}
		} catch (e) {
			error = `Failed load keys: ${e instanceof Error ? e.message : String(e)}`;
			keys = [];
		} finally {
			loadingKeys = false;
		}
	}

	// Select document or signature file
	async function selectFile(type: 'document' | 'signature') {
		error = null;
		verificationResult = null; // Clear previous results
		let title = type === 'document' ? 'Select Original Document' : 'Select Signature File';
		let filters = type === 'signature' ? [{ name: 'Signature', extensions: ['sig'] }] : [];

		try {
			const selected = await openDialog({ multiple: false, title, filters });
			if (selected && typeof selected === 'string') {
				if (type === 'document') documentPath = selected;
				else signaturePath = selected;
			} else {
				// User cancelled
				if (type === 'document') documentPath = null;
				else signaturePath = null;
			}
		} catch (e) {
			error = `Failed select ${type}: ${e instanceof Error ? e.message : String(e)}`;
		}
	}

	// Perform verification
	async function verify() {
		if (!documentPath || !signaturePath || !selectedKeyId) {
			error = 'Please select the document, signature, and key.';
			verificationResult = null;
			return;
		}

		verifying = true;
		error = null;
		verificationResult = null;

		try {
			const result = await invoke<VerificationResult>('verify_signature', {
				documentPath: documentPath,
				signaturePath: signaturePath,
				keyId: selectedKeyId
			});
			verificationResult = result; // Display the result (valid or invalid with reason)
		} catch (e) {
			// Catch unexpected invoke errors
			console.error('Verification invoke failed:', e);
			error = `Verification process failed: ${e instanceof Error ? e.message : String(e)}`;
			verificationResult = null;
		} finally {
			verifying = false;
		}
	}

	onMount(loadKeys);
</script>

<div class="verify-view-container">
	<form onsubmitcapture={verify}>
		<div class="form-group">
			<label for="document" class="form-label">Original Document:</label>
			<div class="file-input-group">
				<button
					type="button"
					class="button"
					onclick={() => selectFile('document')}
					disabled={verifying || loadingKeys}
				>
					<FileText size={18} class="icon" /> Select Document
				</button>
				{#if documentPath}
					<span class="filepath">{documentPath}</span>
				{/if}
			</div>
		</div>
		<div class="form-group">
			<label for="signature" class="form-label">Signature File (.sig):</label>
			<div class="file-input-group">
				<button
					type="button"
					class="button"
					onclick={() => selectFile('signature')}
					disabled={verifying || loadingKeys}
				>
					<KeyRound size={18} class="icon" /> Select Signature
				</button>
				{#if signaturePath}
					<span class="filepath">{signaturePath}</span>
				{/if}
			</div>
		</div>

		<div class="form-group">
			<label for="key" class="form-label">Verifying Key (Public Key):</label>
			{#if loadingKeys}
				<p class="info">Loading keys...</p>
			{:else if keys.length > 0}
				<select
					id="key"
					class="form-control"
					bind:value={selectedKeyId}
					required
					disabled={verifying}
				>
					{#each keys as key (key.keyId)}
						<option value={key.keyId}>{key.name} ({key.algorithm})</option>
					{/each}
				</select>
			{:else}
				<p>
					No keys available to verify with. Please <button
						type="button"
						class="button-link"
						onclick={onBack}>add or generate a key</button
					>.
				</p>
			{/if}
		</div>

		<div class="form-actions">
			<button
				type="submit"
				class="button button-info"
				disabled={verifying ||
					loadingKeys ||
					!documentPath ||
					!signaturePath ||
					!selectedKeyId ||
					keys.length === 0}
			>
				{#if verifying}
					<Loader2 size={18} class="icon" /> Verifying...
				{:else}
					<Search size={18} class="icon" /> Verify Signature
				{/if}
			</button>
			<!-- Optional Back Button -->
			<!-- <button type="button" class="button" on:click={() => dispatch('back')} disabled={verifying}>Back</button> -->
		</div>

		{#if error && !verificationResult}
			<p class="alert alert-danger" style="margin-top: var(--spacing-lg);">{error}</p>
		{/if}

		{#if verificationResult}
			<div class="result alert {verificationResult.isValid ? 'alert-success' : 'alert-danger'}">
				<h4>Verification Result:</h4>
				{#if verificationResult.isValid}
					<p class="result-status">
						<CheckCircle2 size={20} class="icon icon-success" /> Valid Signature
					</p>
					<p>The signature corresponds to the selected document and key.</p>
				{:else}
					<p class="result-status">
						<XCircle size={20} class="icon icon-danger" /> Invalid Signature
					</p>
					{#if verificationResult.errorMessage}
						<p class="error-reason">Reason: {verificationResult.errorMessage}</p>
					{/if}
				{/if}
			</div>
		{/if}
	</form>
</div>

<style>
	.file-input-group {
		/* Same as SignView */
		display: flex;
		align-items: center;
		flex-wrap: wrap;
	}
	.file-input-group button {
		flex-shrink: 0;
	}
	.filepath {
		flex-grow: 1;
		min-width: 200px;
	}

	.form-actions {
		/* Same as SignView */
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

	.result {
		margin-top: var(--spacing-lg);
		/* Base alert styles applied via class */
	}
	.result h4 {
		margin-top: 0;
		margin-bottom: var(--spacing-md);
		font-size: 1.1em;
	}
	.result-status {
		font-weight: bold;
		font-size: 1.1em;
		margin-bottom: var(--spacing-sm);
	}
	.error-reason {
		/* Specific style for the reason text */
		font-size: var(--font-size-sm);
		margin-top: var(--spacing-sm);
	}

	/* Remove redundant global styles */
</style>
