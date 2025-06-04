<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import type { KeyDetails } from '$lib/types';
	import { navTitle } from '$lib/shared';

	let { keyId, onBack } = $props();

	let keyDetails = $state<KeyDetails | null>(null);
	let loading = $state(true); // Initialize as loading
	let error = $state<string | null>(null);
	let copyButtonText = $state('Copy Public Key');

	$effect(() => {
		// Check if keyId is valid before proceeding
		if (!keyId) {
			error = 'No Key ID provided.';
			loading = false;
			keyDetails = null; // Ensure details are cleared
			$navTitle  = `Key Details`;
			return; // Stop the effect execution here
		}

		// Reset state for the new load
		loading = true;
		error = null;
		keyDetails = null; // Clear previous details immediately
		copyButtonText = 'Copy Public Key'; // Reset button text
			$navTitle  = `Loading Key...`;

		// Define the async loading logic inside the effect
		const loadData = async () => {
			try {
				const result = await invoke<KeyDetails>('get_key_details', { keyId: keyId }); // Use keyId directly
				keyDetails = result; // Assign fetched data to $state
				if (keyDetails) {
					$navTitle  = `Key Details: ${keyDetails.info.name}`;
				} else {
					// Should not happen if invoke succeeds, but handle defensively
					error = 'Failed to load key details (empty result).';
					$navTitle  = `Key Details Error`;
				}
			} catch (e) {
				console.error(`KeyDetailsView: Failed load for ${keyId}`, e);
				error = `Failed load: ${e instanceof Error ? e.message : String(e)}`;
				$navTitle  = `Key Loading Error`;
			} finally {
				loading = false; // Update loading state
			}
		};

		// Execute the async function
		loadData();

		// No cleanup function needed for this effect as it only fetches data
	});

	function copyToClipboard(text: string) {
		if (!text) return;
		navigator.clipboard
			.writeText(text)
			.then(() => {
				copyButtonText = 'Copied!'; // Update $state
				setTimeout(() => (copyButtonText = 'Copy Public Key'), 2000);
			})
			.catch((err) => {
				console.error('Failed to copy:', err);
				copyButtonText = 'Copy Failed'; // Update $state
				setTimeout(() => (copyButtonText = 'Copy Public Key'), 2000);
			});
	}
</script>

<div class="key-details-container">
	<button class="button button-link back-button" onclick={onBack}> ← Back to List </button>

	{#if loading}
		<p class="info">Loading key details...</p>
	{:else if error}
		<p class="alert alert-danger">{error}</p>
	{:else if keyDetails}
		<!-- Removed H2, title handled by parent -->
		<dl class="details-list">
			<dt>Name:</dt>
			<dd>{keyDetails.info.name}</dd>

			<dt>Key ID:</dt>
			<dd class="key-id">{keyDetails.info.keyId}</dd>

			<dt>Algorithm:</dt>
			<dd>{keyDetails.info.algorithm}</dd>

			<dt>Created At:</dt>
			<dd>{new Date(keyDetails.info.createdAt).toLocaleString()}</dd>

			<dt>Public Key (PEM):</dt>
			<dd>
				<pre>{keyDetails.publicKeyPem}</pre>
				<button class="button" onclick={() => copyToClipboard(keyDetails!.publicKeyPem)}>
					{copyButtonText}
				</button>
			</dd>
		</dl>
	{:else}
		<p>Key details could not be loaded.</p>
	{/if}
</div>

<style>
	.back-button {
		margin-bottom: var(--spacing-lg);
		font-weight: 500; /* Make it slightly bolder */
	}

	.details-list {
		display: grid;
		grid-template-columns: auto 1fr; /* Label column sized by content */
		gap: var(--spacing-sm) var(--spacing-md);
		margin-top: var(--spacing-md);
		border-top: 1px solid var(--color-border);
		padding-top: var(--spacing-md);
	}
	dt {
		font-weight: 600;
		grid-column: 1;
		color: var(--color-text-muted);
	}
	dd {
		grid-column: 2;
		margin: 0;
		word-break: break-all; /* Break long values */
	}
	.key-id {
		font-family: var(--font-family-mono); /* Use monospace for IDs */
		font-size: var(--font-size-sm);
	}
	dd pre {
		margin-bottom: var(--spacing-sm); /* Space between pre and button */
	}

	/* Ensure .info class exists if used above */
	.info {
		color: var(--color-text-muted);
		font-style: italic;
	}

	/* Remove redundant styles handled globally */
</style>
