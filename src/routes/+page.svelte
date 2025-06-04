<script lang="ts">
	import type { KeyDetails } from '$lib/types';
	import './styles.css'; // Import global styles HERE if not in layout

	// --- Import Child Components ---
	import KeyList from '$lib/components/KeyList.svelte';
	import GenerateKey from '$lib/components/GenerateKey.svelte';
	import KeyDetailsView from '$lib/components/KeyDetailsView.svelte';
	import SignView from '$lib/components/SignView.svelte';
	import VerifyView from '$lib/components/VerifyView.svelte';
	import { navTitle } from '$lib/shared';

	// --- State Management ---
	type View = 'list' | 'generate' | 'details' | 'sign' | 'verify';
	let currentView = $state<View>('list');
	let selectedKeyId = $state<string | null>(null);
	// let title = $state('Key Management');

	// --- Navigation Functions ---
	function showList() {
		currentView = 'list';
		selectedKeyId = null;
		$navTitle = 'Key Management';
	}
	function showGenerate() {
		currentView = 'generate';
		selectedKeyId = null;
		$navTitle = 'Generate New Key';
	}
	function showDetails(keyId: string) {
		selectedKeyId = keyId;
		currentView = 'details';
		$navTitle = 'Key Details';
	}
	function showSign() {
		currentView = 'sign';
		selectedKeyId = null;
		$navTitle = 'Sign Document';
	}
	function showVerify() {
		currentView = 'verify';
		selectedKeyId = null;
		$navTitle = 'Verify Signature';
	}

	// --- Event Handlers ---
	function handleKeyGenerated(details: KeyDetails) {
		showDetails(details.info.keyId);
	}
	// function handleTitleUpdate(newTitle: string) {
	// 	navTitle = newTitle;
	// }
</script>

<div class="app-container">
	<header class="app-header">
		<h1>{$navTitle}</h1>
	</header>

	<nav class="app-nav">
		<button
			class="button nav-button"
			onclick={showList}
			class:active={currentView === 'list' ||
				currentView === 'generate' ||
				currentView === 'details'}>Manage Keys</button
		>
		<button class="button nav-button" onclick={showSign} class:active={currentView === 'sign'}
			>Sign Document</button
		>
		<button class="button nav-button" onclick={showVerify} class:active={currentView === 'verify'}
			>Verify Signature</button
		>
	</nav>

	<main class="content-area">
		{#if currentView === 'list'}
			<KeyList onGenerate={showGenerate} onViewDetails={showDetails} />
		{:else if currentView === 'generate'}
			<GenerateKey onGenerated={handleKeyGenerated} onCancel={showList} />
		{:else if currentView === 'details' && selectedKeyId}
			<KeyDetailsView keyId={selectedKeyId} onBack={showList} />
		{:else if currentView === 'sign'}
			<SignView onBack={showList} />
		{:else if currentView === 'verify'}
			<VerifyView onBack={showList} />
		{/if}
	</main>

	<footer class="app-footer">
		<!-- Optional footer -->
		<p>© {new Date().getFullYear()} Sigust</p>
	</footer>
</div>

<style>
	.app-container {
		display: flex;
		flex-direction: column;
		min-height: 100vh; /* Full viewport height */
		background-color: var(--color-background);
	}

	.app-header {
		background-color: var(--color-dark);
		color: var(--color-light);
		padding: var(--spacing-md) var(--spacing-lg);
		text-align: center;
		flex-shrink: 0; /* Header doesn't shrink */
	}
	.app-header h1 {
		margin: 0;
		font-size: var(--font-size-lg); /* Slightly smaller header */
	}

	.app-nav {
		display: flex;
		justify-content: center;
		background-color: var(--color-background-alt);
		padding: var(--spacing-sm) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
		gap: var(--spacing-sm); /* Spacing between nav items */
		flex-shrink: 0;
		flex-wrap: wrap; /* Allow wrapping on smaller screens */
	}

	/* Style nav buttons differently */
	.nav-button {
		flex-grow: 1; /* Allow buttons to grow */
		max-width: 200px; /* Prevent buttons getting too large */
		background-color: transparent;
		border: 1px solid transparent; /* Maintain layout */
		color: var(--color-text-muted);
		padding: var(--spacing-sm);
		margin: 0; /* Override default button margin */
		transition:
			background-color 0.2s ease,
			color 0.2s ease;
		font-weight: 500;
	}
	.nav-button:hover {
		background-color: rgba(0, 0, 0, 0.05); /* Subtle hover */
		color: var(--color-text);
		border-color: transparent;
	}
	.nav-button.active {
		background-color: var(--color-primary);
		color: white;
		border-color: var(--color-primary-dark);
	}
	.nav-button.active:hover {
		background-color: var(--color-primary-dark);
	}

	.content-area {
		padding: var(--spacing-lg);
		flex-grow: 1; /* Allow content to fill space */
		overflow-y: auto; /* Allow scrolling if content overflows */
		background-color: var(--color-background);
	}

	/* Make container within content area centered and not too wide */
	:global(.content-area > div) {
		/* Target direct child div of content-area */
		max-width: 800px;
		margin-left: auto;
		margin-right: auto;
	}

	.app-footer {
		flex-shrink: 0;
		text-align: center;
		padding: var(--spacing-md);
		background-color: var(--color-background-alt);
		border-top: 1px solid var(--color-border);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}
	.app-footer p {
		margin: 0;
	}
</style>
