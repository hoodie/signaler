<script>
    import { createEventDispatcher } from 'svelte'
    import MessageView from './MessageView.svelte'
    import { messages } from './stores.js'

    const dispatch = createEventDispatcher();

    export let rooms;
    export let sessionId;

    let currentRoom = 'default';

    function send({detail: message}) {
        dispatch('send', { message, currentRoom });
    }

</script>
<code> RoomView </code>

{#each rooms as room}
<button on:click={() => currentRoom = room}>
{room}
</button>
{/each}

<section>
    <MessageView messages={$messages[currentRoom]} me={sessionId} on:submit={send}/>
</section>