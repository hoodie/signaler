<style>
    dl {
        width: 600px;
        overflow: hidden;
        padding: 0;
        margin: 0
    }
    dt {
        float: left;
        width: 50%;
        padding: 0;
        margin: 0
    }
    dd {
        float: left;
        width: 50%;
        padding: 0;
        margin: 0;
    }
</style>
<script>
    import { onMount, onDestroy, createEventDispatcher } from 'svelte'

    import Login from './Login.svelte';
    import RoomView from './RoomView.svelte'
    import { messages } from './stores.js'

    export let session;

    let connected = false
    let sessionId = null;
    let profile = null;
    let authenticated = false;

    $: rooms = [];
    let myrooms = [];
    let participantsByRoom = {};
    let receivedMessagesByRoom = {};

    let messagesByRoom = {}


    session.onAuthenticated.add(() => {
        authenticated = true;
        session.join('default');
        session.join('test');
    });

    session.onRoomList.add(r => rooms = r);
    session.onMyRoomList.add(r=>  myrooms = r );
    session.onProfile.add(p => profile = p)

    session.onMessage.add(({room, message}) => {
        messages.update(messagesByRoom => {
            const roomMessages = [...(messagesByRoom[room]||[]), message];
            return {
                ...messagesByRoom,
                [room]: roomMessages
            }
        });
    });

    async function connect() {
        console.info("connect()")
        session.onWelcome.add(() => { console.debug('/0\\', session) });
        const sessionDescription = await session.connect();
        sessionId = sessionDescription.sessionId
        connected = true;

        session.sendCommand({ type: 'listRooms' });
        session.sendCommand({ type: 'listMyRooms' });
        session.authenticate('jon', 'password')
    }

    function send({detail: {message, currentRoom}}) {
        session.sendMessage(message, currentRoom);
    }

	onMount(async () => {
        console.debug("mounted", session)
        window.session = session
        await connect();
    });
    onDestroy(() => {
        console.debug("destroy", session)
        session.disconnect()
    });

</script>

<h2> SessionView </h2>

<aside>
<dl>
    <dt>connected</dt>
    <dd>{connected}</dd>

    <dt>sessionId</dt>
    <dd>{sessionId}</dd>

    <dt>authenticated</dt>
    <dd>{authenticated}</dd>

    <dt>user</dt>
    <dd>{profile && profile.fullName}</dd>
</dl>
</aside>

{#if !connected}
    <button on:click={connect}> connect </button>
{:else}
    {#if !authenticated}
        <Login on:login={({detail: {username, password}}) => session.authenticate(username, password)} />
    {:else}
        <RoomView 
            sessionId={sessionId}
            rooms={myrooms}
            messagesByRoom={messagesByRoom}
            on:send={send}
        />
    {/if}
{/if}
