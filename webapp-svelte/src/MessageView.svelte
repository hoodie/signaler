<style>
    div {
        outline: 1px solid red;
    }
    .messageList {
        display: grid;
        grid-template-columns: 1fr 3fr 10fr;
        font-size: 0.8em;
        font-family: monospace;
        outline: 1px dashed green;
        max-height: 5em;
        overflow-y: scroll;
    }

    .messageList .timestamp {
        font-weight: bold;
    }

    .messageList .sender {
        font-size: 0.8em;
    }
    .messageList .sender.me {
        font-weight: bold;
        font-size: 1.2em;
    }
</style>

<script>
    import { afterUpdate, beforeUpdate, createEventDispatcher } from 'svelte'

    const dispatch = createEventDispatcher();

    export let messages = [];
    export let me;

    let end;
    let newMessage = "";

    const prettyTime = (date) => `${date.getHours()}:${date.getMinutes()}`
    const senderName = message => message.sender === me ? 'me' : message.senderName;

    const handleSubmit = ({key}) => key === 'Enter' && submit();

    function submit() {
        dispatch('submit', newMessage);
        newMessage = "";
    }

	beforeUpdate(() => {
        end && end.scrollIntoView({behavior: 'smooth'});
        console.log({end: end})
	})

</script>

<code> MessageView </code>

<div class="messageList">
{#each messages as message}
    <span class="timestamp"> {prettyTime(message.received)} </span>
    <span class={`sender ${senderName(message)}`}>{senderName(message)}</span>
    <span class="content" bind:this={end} > {message.content} </span>
{/each}
</div>

<input type="text" bind:value={newMessage} on:keydown={handleSubmit}/>