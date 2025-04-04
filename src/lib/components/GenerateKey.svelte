<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import {
		supportedAlgorithmsForGeneration,
		type KeyDetails,
		type SignatureAlgorithm
	} from '$lib/types';
	import { Loader2 } from 'lucide-svelte';

	let { onGenerated, onCancel } = $props();

	let keyName = $state('');
	let selectedAlgorithm = $state<SignatureAlgorithm>(supportedAlgorithmsForGeneration[0].value);
	let password = $state('');
	let confirmPassword = $state('');
	let loading = $state(false);
	let error = $state<string | null>(null);

	async function generateKey() {
		if (password !== confirmPassword) {
			error = 'Passwords do not match.';
			return;
		}
		if (!keyName.trim()) {
			error = 'Key name cannot be empty.';
			return;
		}
		if (!password) {
			error = 'Password cannot be empty.';
			return;
		}

		loading = true;
		error = null;

		try {
			const newKeyDetails = await invoke<KeyDetails>('generate_key_pair', {
				name: keyName.trim(),
				algStr: selectedAlgorithm,
				password: password
			});
			onGenerated(newKeyDetails)
		} catch (e) {
			console.error('GenerateKey: Failed generation', e);
			error = `Generation failed: ${e instanceof Error ? e.message : String(e)}`;
		} finally {
			loading = false;
		}
	}
</script>

<!-- Component's main container div -->
<div class="generate-key-container">
	<!-- Removed H2 -->
	<form onsubmitcapture={generateKey}>
		<div class="form-group">
			<label for="keyName" class="form-label">Key Name:</label>
			<input
				id="keyName"
				type="text"
				class="form-control"
				bind:value={keyName}
				required
				disabled={loading}
				placeholder="e.g., My Work Key"
			/>
		</div>

		<div class="form-group">
			<label for="algorithm" class="form-label">Signature Algorithm:</label>
			<select
				id="algorithm"
				class="form-control"
				bind:value={selectedAlgorithm}
				required
				disabled={loading}
			>
				{#each supportedAlgorithmsForGeneration as alg (alg.value)}
					<option value={alg.value}>{alg.label}</option>
				{/each}
			</select>
		</div>

		<div class="form-group">
			<label for="password" class="form-label">Password:</label>
			<input
				id="password"
				type="password"
				class="form-control"
				bind:value={password}
				required
				disabled={loading}
			/>
			<small class="form-text">Choose a strong password to protect your private key.</small>
		</div>
		<div class="form-group">
			<label for="confirmPassword" class="form-label">Confirm Password:</label>
			<input
				id="confirmPassword"
				type="password"
				class="form-control"
				bind:value={confirmPassword}
				required
				disabled={loading}
			/>
		</div>

		{#if error}
			<p class="alert alert-danger">{error}</p>
			<!-- Use alert classes -->
		{/if}

		<div class="form-actions">
			<button type="submit" class="button button-success" disabled={loading}>
				{#if loading}
					<Loader2 size={18} class="icon" /> Generating...{:else}Generate Key{/if}
			</button>
			<button type="button" class="button" onclick={onCancel} disabled={loading}>
				Cancel
			</button>
		</div>
	</form>
</div>

<style>
	.form-actions {
		margin-top: var(--spacing-lg);
		padding-top: var(--spacing-md);
		border-top: 1px solid var(--color-border); /* Separator */
		display: flex;
		gap: var(--spacing-sm);
	}

	/* Simple spinner animation */
	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
