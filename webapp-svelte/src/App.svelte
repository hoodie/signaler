<style>
.SessionView {
	outline: 1px dashed orange;
}
</style>
<script>
	import { onMount } from 'svelte'
	import { Session } from '../../client-lib/lib';

	import SessionView  from './SessionView.svelte';
	export let url;

    let session = new Session(url);
    session.onConnectionClose.add(event => {
		console.error(event);
		session = null
    });

	onMount(() => {
		console.debug("creating session", session)
	})
</script>


{#if session}
<section class="SessionView">
	<SessionView session={session}/>
</section>
{:else}
<strong>no session</strong>
{/if}