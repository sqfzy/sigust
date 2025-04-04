<script lang="ts">
	import { onMount} from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { KeyInfo } from '$lib/types';
	import { Plus, RefreshCcw } from 'lucide-svelte'; // RefreshCcw is a common refresh icon

	let { onGenerate, onViewDetails } = $props();

	let keys = $state<KeyInfo[]>([]); // Initialize with empty array
	let loading = $state(true); // Start in loading state
	let error = $state<string | null>(null);

	async function loadKeys() {
		loading = true;
		error = null;
		try {
			keys = await invoke<KeyInfo[]>('list_keys');
		} catch (e) {
			console.error('KeyList: Failed load keys', e);
			error = `Failed load: ${e instanceof Error ? e.message : String(e)}`;
		} finally {
			loading = false;
		}
	}

	onMount(loadKeys);
</script>

<!-- Component's main container div -->
<div class="key-list-container">
	<!-- Removed H2, title handled by parent -->
	<div class="action-bar">
		<button class="button button-primary" onclick={onGenerate}>
			<Plus size={18} class="icon" /> Generate New Key Pair
		</button>
		<button class="button" onclick={loadKeys} disabled={loading} title="Refresh key list">
			<RefreshCcw size={18} class="icon" />
			{#if !loading}Refresh{:else}Refreshing...{/if}
		</button>
	</div>

	{#if loading}
		<p class="info">Loading keys...</p>
		<!-- Use info class or similar -->
	{:else if error}
		<p class="alert alert-danger">{error}</p>
		<!-- Use alert classes -->
	{:else if keys.length === 0}
		<p>No keys found. Click "Generate New Key Pair" to get started!</p>
	{:else}
		<h3 class="key-list-heading">Available Keys:</h3>
		<ul class="key-list">
			{#each keys as key (key.keyId)}
				<li class="key-list-item">
					<button
						class="button-link key-name"
						onclick={() => onViewDetails(key.keyId)}
					>
						<strong>{key.name}</strong>
					</button>
					<div class="key-meta">
						<span class="key-algorithm">{key.algorithm}</span> |
						<span class="key-date">Created: {new Date(key.createdAt).toLocaleString()}</span>
						<span class="key-id-short">ID: {key.keyId.substring(0, 8)}...</span>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.action-bar {
		margin-bottom: var(--spacing-lg);
		display: flex;
		gap: var(--spacing-sm);
		flex-wrap: wrap; /* Allow buttons to wrap */
	}

	.key-list-heading {
		margin-top: var(--spacing-lg);
		margin-bottom: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		padding-bottom: var(--spacing-sm);
	}

	.key-list {
		list-style: none;
		padding: 0;
	}

	.key-list-item {
		margin-bottom: var(--spacing-md);
		padding: var(--spacing-sm);
		border-radius: var(--border-radius);
		background-color: var(--color-background-alt);
		border: 1px solid var(--color-border);
		transition: background-color 0.2s ease;
	}
	.key-list-item:hover {
		background-color: #e9ecef; /* Slightly darker hover */
	}

	.key-name {
		font-size: 1.1em; /* Make name slightly larger */
		display: block; /* Ensure it takes full width for easier clicking */
		margin-bottom: var(--spacing-xs);
		text-align: left; /* Align link-button text left */
	}
	.key-name strong {
		font-weight: 600;
	}

	.key-meta {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		display: flex;
		flex-wrap: wrap; /* Allow meta info to wrap */
		gap: var(--spacing-sm);
	}
	.key-algorithm,
	.key-date,
	.key-id-short {
		white-space: nowrap; /* Prevent individual items from wrapping */
	}

	/* Ensure .info class exists if used above */
	.info {
		color: var(--color-text-muted);
		font-style: italic;
	}

	/* Remove redundant styles that are now global */
</style>
