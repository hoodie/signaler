<script>

    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    let username = "";
    let password = "";

    $: valid = password.length > 0

    const handleKey= ({key}) => key === 'Enter' && submit();

    const submit = () => valid ? dispatch('login', {username, password}) : alert("invalid password");


</script>

<input type="text"     placeholder="username" bind:value={username}/>
<input type="password" placeholder="password" bind:value={password} on:keypress={handleKey} />
<button on:click={submit} disabled={!valid}>login</button>
